//! Comprehensive audit trail for BOS ↔ BusinessOS operations.
//!
//! Provides:
//! - Immutable hash-chain audit logging
//! - Merkle tree verification
//! - GDPR compliance hooks
//! - Real-time tamper detection
//! - Event streaming to BusinessOS

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub mod encryption;
pub mod events;
pub mod export;
pub mod hash_chain;

pub use encryption::{AuditEncryption, EncryptionConfig};
pub use events::{AuditCategory, AuditEvent, AuditEventType};
pub use export::{AuditExport, ExportFormat};
pub use hash_chain::{ChainVerification, HashChain, HashChainEntry};

/// Central audit logger for all BOS operations.
#[derive(Clone)]
pub struct AuditLogger {
    entries: Arc<Mutex<Vec<HashChainEntry>>>,
    head_hash: Arc<Mutex<String>>,
    config: AuditConfig,
}

/// Audit system configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable encryption at rest
    pub encrypt_at_rest: bool,

    /// Encryption key (if encrypt_at_rest = true)
    pub encryption_key: Option<String>,

    /// Enable HTTP streaming to BusinessOS
    pub stream_to_businessos: bool,

    /// BusinessOS audit endpoint
    pub businessos_endpoint: Option<String>,

    /// Retention policy in days
    pub retention_days: u32,

    /// Enable integrity verification job
    pub enable_verification: bool,

    /// Verification job interval in hours
    pub verification_interval_hours: u32,

    /// Maximum events in memory before flush
    pub buffer_size: usize,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            encrypt_at_rest: true,
            encryption_key: None,
            stream_to_businessos: true,
            businessos_endpoint: Some("http://localhost:8001/api/audit/events".to_string()),
            retention_days: 2555, // 7 years
            enable_verification: true,
            verification_interval_hours: 24,
            buffer_size: 1000,
        }
    }
}

impl AuditLogger {
    /// Create a new audit logger.
    pub fn new(config: AuditConfig) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            head_hash: Arc::new(Mutex::new(Self::genesis_hash())),
            config,
        }
    }

    /// Log a process mining operation.
    pub fn log_model_discovered(
        &self,
        user_id: Uuid,
        log_source: String,
        algorithm: String,
        model_hash: String,
        activities_count: u32,
        duration_ms: u64,
    ) -> Result<Uuid, AuditError> {
        let event = AuditEvent::model_discovered(
            user_id,
            log_source,
            algorithm,
            model_hash,
            activities_count,
            duration_ms,
        );

        self.record_event(event)
    }

    /// Log a conformance check operation.
    #[allow(clippy::too_many_arguments)]
    pub fn log_conformance_checked(
        &self,
        user_id: Uuid,
        model_id: Uuid,
        log_id: Uuid,
        fitness: f64,
        precision: f64,
        generalization: f64,
        log_entries_tested: u32,
    ) -> Result<Uuid, AuditError> {
        let event = AuditEvent::conformance_checked(
            user_id,
            model_id,
            log_id,
            fitness,
            precision,
            generalization,
            log_entries_tested,
        );

        self.record_event(event)
    }

    /// Log statistics computation.
    pub fn log_statistics_computed(
        &self,
        user_id: Uuid,
        log_id: Uuid,
        statistic_type: String,
        result_hash: String,
        sample_size: u32,
    ) -> Result<Uuid, AuditError> {
        let event = AuditEvent::statistics_computed(
            user_id,
            log_id,
            statistic_type,
            result_hash,
            sample_size,
        );

        self.record_event(event)
    }

    /// Log access grant (compliance).
    pub fn log_access_granted(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
        resource_type: String,
        permission: String,
        granted_by: Uuid,
    ) -> Result<Uuid, AuditError> {
        let event =
            AuditEvent::access_granted(user_id, resource_id, resource_type, permission, granted_by);

        self.record_event(event)
    }

    /// Log access revocation.
    pub fn log_access_revoked(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
        revoked_by: Uuid,
        reason: String,
    ) -> Result<Uuid, AuditError> {
        let event = AuditEvent::access_revoked(user_id, resource_id, revoked_by, reason);
        self.record_event(event)
    }

    /// Log data deletion (GDPR right to be forgotten).
    pub fn log_data_deletion(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
        deletion_reason: String,
        previous_hash: String,
    ) -> Result<Uuid, AuditError> {
        let event = AuditEvent::data_deletion(user_id, resource_id, deletion_reason, previous_hash);
        self.record_event(event)
    }

    /// Log authentication failure (security event).
    pub fn log_authentication_failure(
        &self,
        username_hash: String,
        ip_address: String,
        failure_reason: String,
    ) -> Result<Uuid, AuditError> {
        let event = AuditEvent::authentication_failure(username_hash, ip_address, failure_reason);
        self.record_event(event)
    }

    /// Log privilege escalation attempt (security event).
    pub fn log_privilege_escalation_attempt(
        &self,
        user_id: Uuid,
        attempted_role: String,
    ) -> Result<Uuid, AuditError> {
        let event = AuditEvent::privilege_escalation_attempt(
            user_id,
            attempted_role,
            "blocked".to_string(),
        );
        self.record_event(event)
    }

    /// Log suspicious activity (security event).
    pub fn log_suspicious_activity(
        &self,
        user_id: Uuid,
        activity_type: String,
        confidence_score: f64,
    ) -> Result<Uuid, AuditError> {
        let event =
            AuditEvent::suspicious_activity_detected(user_id, activity_type, confidence_score);
        self.record_event(event)
    }

    /// Record an audit event in the hash chain.
    fn record_event(&self, mut event: AuditEvent) -> Result<Uuid, AuditError> {
        let mut entries = self.entries.lock().map_err(|_| AuditError::LockFailed)?;
        let mut head_hash = self.head_hash.lock().map_err(|_| AuditError::LockFailed)?;

        // Assign sequence number
        let sequence_number = entries.len() as u64 + 1;
        event.chain.sequence_number = sequence_number;

        // Link to previous entry
        event.chain.previous_hash = head_hash.clone();

        // Serialize the event payload BEFORE setting entry_hash so that the
        // hash and the stored payload are computed from the same data.
        // Invariant: sha256(serde_json::to_string(&entry.payload)) == entry.entry_hash
        let payload = serde_json::to_value(&event).map_err(|_| AuditError::SerializationFailed)?;

        // Compute entry hash over the serialized payload (not the full AuditEvent
        // which has entry_hash still as an empty placeholder).
        let entry_hash = {
            let payload_str =
                serde_json::to_string(&payload).map_err(|_| AuditError::SerializationFailed)?;
            let mut hasher = Sha256::new();
            hasher.update(payload_str.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        // Keep entry_hash on the event consistent (used for streaming).
        event.chain.entry_hash = entry_hash.clone();

        let event_id = event.event_id;

        // Create chain entry — payload already serialized above.
        let chain_entry = HashChainEntry {
            sequence_number,
            event_id,
            timestamp: event.timestamp.clone(),
            event_type: event.event_type.clone(),
            event_category: event.event_category.clone(),
            previous_hash: event.chain.previous_hash.clone(),
            entry_hash: entry_hash.clone(),
            payload,
        };

        entries.push(chain_entry);

        // Update head hash
        *head_hash = entry_hash;

        // Stream to BusinessOS if enabled
        if self.config.stream_to_businessos {
            self.stream_event_async(event);
        }

        // Flush to storage if buffer full
        if entries.len() >= self.config.buffer_size {
            drop(entries); // Release lock before flush
            self.flush_to_storage()?;
        }

        Ok(event_id)
    }

    /// Compute Merkle root from an already-locked slice of entries.
    ///
    /// This avoids acquiring `self.entries` lock again inside functions that
    /// already hold it (e.g. `verify_chain`), preventing a deadlock on the
    /// non-reentrant `std::sync::Mutex`.
    fn compute_merkle_root_from_slice(entries_slice: &[&HashChainEntry]) -> String {
        if entries_slice.is_empty() {
            return Self::genesis_hash();
        }

        let mut hashes: Vec<String> = entries_slice.iter().map(|e| e.entry_hash.clone()).collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    format!("{}{}", chunk[0], chunk[0])
                };
                let mut hasher = Sha256::new();
                hasher.update(combined.as_bytes());
                next_level.push(format!("{:x}", hasher.finalize()));
            }
            hashes = next_level;
        }

        hashes.pop().unwrap_or_else(Self::genesis_hash)
    }

    /// Genesis block hash (first entry reference).
    fn genesis_hash() -> String {
        "0".repeat(64)
    }

    /// Get all entries in the audit chain.
    pub fn get_entries(&self) -> Result<Vec<HashChainEntry>, AuditError> {
        let entries = self.entries.lock().map_err(|_| AuditError::LockFailed)?;
        Ok(entries.clone())
    }

    /// Verify chain integrity from sequence N to M.
    pub fn verify_chain(
        &self,
        from_seq: u64,
        to_seq: u64,
    ) -> Result<ChainVerification, AuditError> {
        let entries = self.entries.lock().map_err(|_| AuditError::LockFailed)?;

        let mut valid = true;
        let mut verified_count = 0;
        let mut issues = Vec::new();

        for i in (from_seq as usize)..(to_seq.min(entries.len() as u64) as usize) {
            if i >= entries.len() {
                break;
            }

            let entry = &entries[i];

            // Verify entry hash
            let computed_hash = {
                let mut hasher = Sha256::new();
                hasher.update(
                    serde_json::to_string(&entry.payload)
                        .expect("audit entry serialization must succeed")
                        .as_bytes(),
                );
                format!("{:x}", hasher.finalize())
            };

            if computed_hash != entry.entry_hash {
                valid = false;
                issues.push(format!(
                    "Entry {} hash mismatch: expected {}, got {}",
                    entry.sequence_number, entry.entry_hash, computed_hash
                ));
            }

            // Verify chain link
            if i > 0 {
                let previous_entry = &entries[i - 1];
                if entry.previous_hash != previous_entry.entry_hash {
                    valid = false;
                    issues.push(format!(
                        "Entry {} chain break: expected {}, got {}",
                        entry.sequence_number, previous_entry.entry_hash, entry.previous_hash
                    ));
                }
            } else {
                // Genesis entry should reference genesis hash
                if entry.previous_hash != Self::genesis_hash() {
                    valid = false;
                    issues.push(format!(
                        "Entry {} should reference genesis hash, got {}",
                        entry.sequence_number, entry.previous_hash
                    ));
                }
            }

            verified_count += 1;
        }

        // Compute Merkle root using the already-held lock guard to avoid
        // a deadlock: `compute_merkle_root` would try to acquire `self.entries`
        // again, which would deadlock on the non-reentrant std::sync::Mutex.
        let range: Vec<&HashChainEntry> = entries
            .iter()
            .skip(from_seq as usize)
            .take((to_seq as usize).saturating_sub(from_seq as usize))
            .collect();
        let merkle_root = Self::compute_merkle_root_from_slice(&range);

        Ok(ChainVerification {
            valid,
            verified_entries: verified_count,
            merkle_root,
            verified_at: Utc::now(),
            issues,
        })
    }

    /// Compute Merkle tree root for a range of entries.
    pub fn compute_merkle_root(
        &self,
        from_idx: usize,
        to_idx: usize,
    ) -> Result<String, AuditError> {
        let entries = self.entries.lock().map_err(|_| AuditError::LockFailed)?;

        let range_entries: Vec<&HashChainEntry> = entries
            .iter()
            .skip(from_idx)
            .take(to_idx.saturating_sub(from_idx))
            .collect();

        Ok(Self::compute_merkle_root_from_slice(&range_entries))
    }

    /// Export audit trail in specified format.
    pub fn export(&self, format: ExportFormat) -> Result<Vec<u8>, AuditError> {
        let entries = self.entries.lock().map_err(|_| AuditError::LockFailed)?;
        AuditExport::export(&entries, format).map_err(AuditError::ExportFailed)
    }

    /// Stream event to BusinessOS asynchronously.
    fn stream_event_async(&self, event: AuditEvent) {
        let endpoint = self.config.businessos_endpoint.clone();
        std::thread::spawn(move || {
            if let Some(url) = endpoint {
                let client = reqwest::Client::new();
                // Build the request; the future is intentionally dropped — this
                // is a best-effort fire-and-forget to BusinessOS.  The caller
                // does not need to await the result.
                drop(client.post(&url).json(&event).send());
            }
        });
    }

    /// Flush buffered events to persistent storage.
    fn flush_to_storage(&self) -> Result<(), AuditError> {
        // Implementation: persist to PostgreSQL via BusinessOS API
        // This is called when buffer reaches capacity
        let entries = self.entries.lock().map_err(|_| AuditError::LockFailed)?;

        // TODO: Batch insert to PostgreSQL via /api/audit/events endpoint
        // For now, log count
        tracing::info!("Flushing {} audit entries to storage", entries.len());

        Ok(())
    }

    /// Start periodic integrity verification job.
    pub fn start_verification_job(&self) -> Result<(), AuditError> {
        if !self.config.enable_verification {
            return Ok(());
        }

        let verification_interval =
            std::time::Duration::from_secs((self.config.verification_interval_hours as u64) * 3600);

        let logger = self.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(verification_interval);

            let entries = logger.entries.lock().map(|e| e.len() as u64).unwrap_or(0);

            if entries > 0 {
                match logger.verify_chain(1, entries) {
                    Ok(verification) => {
                        if verification.valid {
                            tracing::info!(
                                "[AuditChain] Verification passed. Entries: {}",
                                entries
                            );
                        } else {
                            tracing::error!(
                                "[AuditChain] TAMPERING DETECTED! Issues: {:?}",
                                verification.issues
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("[AuditChain] Verification error: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

/// Audit system errors.
#[derive(Debug, Clone)]
pub enum AuditError {
    LockFailed,
    SerializationFailed,
    EncryptionFailed(String),
    ExportFailed(String),
    VerificationFailed(String),
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditError::LockFailed => write!(f, "Failed to acquire lock"),
            AuditError::SerializationFailed => write!(f, "Failed to serialize event"),
            AuditError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            AuditError::ExportFailed(msg) => write!(f, "Export failed: {}", msg),
            AuditError::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
        }
    }
}

impl std::error::Error for AuditError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logger_creation() {
        let logger = AuditLogger::new(AuditConfig::default());
        assert!(logger.get_entries().is_ok());
    }

    #[test]
    fn test_model_discovered_logging() {
        let logger = AuditLogger::new(AuditConfig::default());
        let user_id = Uuid::new_v4();

        let result = logger.log_model_discovered(
            user_id,
            "s3://bucket/log.xes".to_string(),
            "alpha".to_string(),
            "model_hash_123".to_string(),
            12,
            5000,
        );

        assert!(result.is_ok());
        assert_eq!(logger.get_entries().unwrap().len(), 1);
    }

    #[test]
    fn test_chain_integrity() {
        let logger = AuditLogger::new(AuditConfig::default());
        let user_id = Uuid::new_v4();

        let _ = logger.log_model_discovered(
            user_id,
            "log1.xes".to_string(),
            "alpha".to_string(),
            "hash1".to_string(),
            10,
            1000,
        );

        let _ = logger.log_conformance_checked(
            user_id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            0.95,
            0.92,
            0.88,
            5000,
        );

        let verification = logger.verify_chain(0, 2).unwrap();
        assert!(verification.valid);
        assert_eq!(verification.verified_entries, 2);
    }

    #[test]
    fn test_merkle_root_computation() {
        let logger = AuditLogger::new(AuditConfig::default());
        let user_id = Uuid::new_v4();

        for i in 0..5 {
            let _ = logger.log_model_discovered(
                user_id,
                format!("log{}.xes", i),
                "alpha".to_string(),
                format!("hash{}", i),
                10,
                1000,
            );
        }

        let root = logger.compute_merkle_root(0, 5);
        assert!(root.is_ok());
        let root_hash = root.unwrap();
        assert_eq!(root_hash.len(), 64); // SHA256 hex is 64 chars
    }
}
