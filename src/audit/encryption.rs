//! Encryption for audit events at rest.
//!
//! Implements AES-256-GCM (authenticated encryption with additional data).
//! All audit events can be encrypted before storage.

use serde::{Deserialize, Serialize};

/// Encryption configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Master encryption key (32 bytes for AES-256)
    pub master_key: Vec<u8>,

    /// Key rotation interval in days
    pub rotation_interval_days: u32,

    /// Current key version
    pub key_version: u32,

    /// Algorithm identifier
    pub algorithm: String,
}

impl EncryptionConfig {
    /// Create a new encryption config with a random master key.
    pub fn new() -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let master_key: Vec<u8> = (0..32).map(|_| rng.gen()).collect();

        Self {
            master_key,
            rotation_interval_days: 90,
            key_version: 1,
            algorithm: "AES-256-GCM".to_string(),
        }
    }

    /// Create from a hex-encoded key string.
    pub fn from_hex(hex_key: &str) -> Result<Self, String> {
        let master_key =
            hex::decode(hex_key).map_err(|e| format!("Failed to decode hex key: {}", e))?;

        if master_key.len() != 32 {
            return Err(format!(
                "Master key must be 32 bytes (AES-256), got {}",
                master_key.len()
            ));
        }

        Ok(Self {
            master_key,
            rotation_interval_days: 90,
            key_version: 1,
            algorithm: "AES-256-GCM".to_string(),
        })
    }

    /// Export master key as hex string (for secure storage).
    pub fn to_hex(&self) -> String {
        hex::encode(&self.master_key)
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit event encryption/decryption service.
pub struct AuditEncryption {
    config: EncryptionConfig,
}

impl AuditEncryption {
    /// Create a new encryption service.
    pub fn new(config: EncryptionConfig) -> Self {
        Self { config }
    }

    /// Encrypt an audit event payload.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedPayload, String> {
        use aes_gcm::{
            aead::{Aead, AeadCore, KeyInit},
            Aes256Gcm, Key,
        };

        // Generate random nonce (96 bits for GCM) using the AEAD-safe RNG
        let nonce = <Aes256Gcm as AeadCore>::generate_nonce(&mut aes_gcm::aead::OsRng);

        // Create cipher
        let key = Key::<Aes256Gcm>::from_slice(&self.config.master_key);
        let cipher = Aes256Gcm::new(key);

        // Encrypt with key version as additional authenticated data
        let aad = format!("key_version:{}", self.config.key_version).into_bytes();
        let ciphertext = cipher
            .encrypt(
                &nonce,
                aes_gcm::aead::Payload {
                    msg: plaintext,
                    aad: &aad,
                },
            )
            .map_err(|e| format!("Encryption failed: {}", e))?;

        Ok(EncryptedPayload {
            ciphertext: hex::encode(&ciphertext),
            nonce: hex::encode(nonce.as_slice()),
            key_version: self.config.key_version,
            algorithm: self.config.algorithm.clone(),
        })
    }

    /// Decrypt an audit event payload.
    pub fn decrypt(&self, encrypted: &EncryptedPayload) -> Result<Vec<u8>, String> {
        use aes_gcm::{
            aead::generic_array::GenericArray,
            aead::{Aead, KeyInit},
            Aes256Gcm, Key,
        };

        // Verify key version matches
        if encrypted.key_version != self.config.key_version {
            return Err(format!(
                "Key version mismatch: encrypted with v{}, current v{}",
                encrypted.key_version, self.config.key_version
            ));
        }

        // Decode nonce and ciphertext from hex
        let nonce_bytes =
            hex::decode(&encrypted.nonce).map_err(|e| format!("Failed to decode nonce: {}", e))?;
        let ciphertext_bytes = hex::decode(&encrypted.ciphertext)
            .map_err(|e| format!("Failed to decode ciphertext: {}", e))?;

        if nonce_bytes.len() != 12 {
            return Err(format!("Invalid nonce length: {}", nonce_bytes.len()));
        }

        let nonce = GenericArray::clone_from_slice(&nonce_bytes);

        // Create cipher
        let key = Key::<Aes256Gcm>::from_slice(&self.config.master_key);
        let cipher = Aes256Gcm::new(key);

        // Decrypt with key version as additional authenticated data
        let aad = format!("key_version:{}", encrypted.key_version).into_bytes();
        let plaintext = cipher
            .decrypt(
                &nonce,
                aes_gcm::aead::Payload {
                    msg: &ciphertext_bytes,
                    aad: &aad,
                },
            )
            .map_err(|e| format!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Encrypt a JSON object.
    pub fn encrypt_json(&self, obj: &serde_json::Value) -> Result<EncryptedPayload, String> {
        let json_bytes =
            serde_json::to_vec(obj).map_err(|e| format!("JSON serialization failed: {}", e))?;
        self.encrypt(&json_bytes)
    }

    /// Decrypt to a JSON object.
    pub fn decrypt_json(&self, encrypted: &EncryptedPayload) -> Result<serde_json::Value, String> {
        let plaintext = self.decrypt(encrypted)?;
        serde_json::from_slice(&plaintext)
            .map_err(|e| format!("JSON deserialization failed: {}", e))
    }

    /// Get encryption configuration (for key rotation).
    pub fn config(&self) -> &EncryptionConfig {
        &self.config
    }

    /// Rotate encryption key (generates new key version).
    pub fn rotate_key(&mut self, new_master_key: Vec<u8>) -> Result<(), String> {
        if new_master_key.len() != 32 {
            return Err(format!(
                "New master key must be 32 bytes, got {}",
                new_master_key.len()
            ));
        }

        self.config.master_key = new_master_key;
        self.config.key_version += 1;

        tracing::info!(
            "[AuditEncryption] Key rotated to version {}",
            self.config.key_version
        );

        Ok(())
    }
}

/// Encrypted audit event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    /// Hex-encoded ciphertext
    pub ciphertext: String,

    /// Hex-encoded nonce
    pub nonce: String,

    /// Key version used for encryption
    pub key_version: u32,

    /// Algorithm identifier
    pub algorithm: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_config_creation() {
        let config = EncryptionConfig::new();
        assert_eq!(config.master_key.len(), 32);
        assert_eq!(config.key_version, 1);
    }

    #[test]
    fn test_hex_roundtrip() {
        let config = EncryptionConfig::new();
        let hex = config.to_hex();
        let restored = EncryptionConfig::from_hex(&hex).unwrap();
        assert_eq!(config.master_key, restored.master_key);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let config = EncryptionConfig::new();
        let encryption = AuditEncryption::new(config);

        let plaintext = b"Hello, World!";
        let encrypted = encryption.encrypt(plaintext).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_encrypt_decrypt_json() {
        let config = EncryptionConfig::new();
        let encryption = AuditEncryption::new(config);

        let original = serde_json::json!({
            "event_type": "test",
            "data": {
                "field1": "value1",
                "field2": 42
            }
        });

        let encrypted = encryption.encrypt_json(&original).unwrap();
        let decrypted = encryption.decrypt_json(&encrypted).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_tamper_detection() {
        let config = EncryptionConfig::new();
        let encryption = AuditEncryption::new(config);

        let plaintext = b"Secret data";
        let mut encrypted = encryption.encrypt(plaintext).unwrap();

        // Tamper with ciphertext
        let mut ciphertext_bytes = hex::decode(&encrypted.ciphertext).unwrap();
        if !ciphertext_bytes.is_empty() {
            ciphertext_bytes[0] ^= 0xFF; // Flip bits
            encrypted.ciphertext = hex::encode(&ciphertext_bytes);
        }

        // Decryption should fail
        assert!(encryption.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_key_rotation() {
        let config = EncryptionConfig::new();
        let mut encryption = AuditEncryption::new(config);

        let plaintext = b"Original data";
        let encrypted_v1 = encryption.encrypt(plaintext).unwrap();
        assert_eq!(encrypted_v1.key_version, 1);

        // Rotate key
        let new_key: Vec<u8> = (0..32).collect();
        encryption.rotate_key(new_key).unwrap();
        assert_eq!(encryption.config().key_version, 2);

        // Cannot decrypt v1 with v2 key
        let decryption_result = encryption.decrypt(&encrypted_v1);
        assert!(decryption_result.is_err());
    }
}
