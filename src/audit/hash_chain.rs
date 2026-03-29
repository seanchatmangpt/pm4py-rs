//! Hash chain cryptographic integrity verification.
//!
//! Implements tamper-evident audit logging using SHA256 hash chains.
//! Each entry references the previous entry's hash, creating an immutable chain.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Single entry in the hash chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashChainEntry {
    /// Sequence number (primary ordering)
    pub sequence_number: u64,

    /// Unique event ID
    pub event_id: Uuid,

    /// ISO8601 timestamp
    pub timestamp: String,

    /// Event type name
    pub event_type: String,

    /// Event category
    pub event_category: String,

    /// Hash of previous entry (chain link)
    pub previous_hash: String,

    /// SHA256 hash of this entry
    pub entry_hash: String,

    /// Complete event payload
    pub payload: serde_json::Value,
}

/// Complete hash chain for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashChain {
    /// Session identifier
    pub session_id: Uuid,

    /// All chain entries
    pub entries: Vec<HashChainEntry>,

    /// Hash of latest entry (head)
    pub head_hash: String,

    /// Merkle root hash for verification
    pub merkle_root: String,

    /// Integrity verified flag
    pub integrity_verified: bool,

    /// Last verification time
    pub verified_at: Option<DateTime<Utc>>,
}

/// Chain integrity verification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerification {
    /// Whether chain is valid (no tampering detected)
    pub valid: bool,

    /// Number of entries verified
    pub verified_entries: u64,

    /// Merkle root hash
    pub merkle_root: String,

    /// Verification timestamp
    pub verified_at: DateTime<Utc>,

    /// Issues found (if any)
    pub issues: Vec<String>,
}

impl HashChainEntry {
    /// Create a new chain entry from an audit event.
    pub fn from_event(
        sequence_number: u64,
        event_id: Uuid,
        timestamp: String,
        event_type: String,
        event_category: String,
        previous_hash: String,
        payload: serde_json::Value,
    ) -> Self {
        let entry_hash = Self::compute_hash(
            sequence_number,
            &timestamp,
            &event_type,
            &previous_hash,
            &payload,
        );

        Self {
            sequence_number,
            event_id,
            timestamp,
            event_type,
            event_category,
            previous_hash,
            entry_hash,
            payload,
        }
    }

    /// Compute SHA256 hash for this entry.
    fn compute_hash(
        sequence: u64,
        timestamp: &str,
        event_type: &str,
        previous_hash: &str,
        payload: &serde_json::Value,
    ) -> String {
        use sha2::{Digest, Sha256};

        let content = format!(
            "{}|{}|{}|{}|{}",
            sequence,
            timestamp,
            event_type,
            previous_hash,
            serde_json::to_string(&payload).expect("hash_chain entry serialization must succeed")
        );

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify this entry's hash is correct.
    pub fn verify_hash(&self) -> bool {
        let computed = Self::compute_hash(
            self.sequence_number,
            &self.timestamp,
            &self.event_type,
            &self.previous_hash,
            &self.payload,
        );

        computed == self.entry_hash
    }

    /// Verify chain link to next entry.
    pub fn verify_link(&self, next_entry: &HashChainEntry) -> bool {
        next_entry.previous_hash == self.entry_hash
    }
}

impl HashChain {
    /// Create a new empty hash chain.
    pub fn new(session_id: Uuid) -> Self {
        Self {
            session_id,
            entries: Vec::new(),
            head_hash: "0".repeat(64), // Genesis hash
            merkle_root: "0".repeat(64),
            integrity_verified: false,
            verified_at: None,
        }
    }

    /// Add an entry to the chain.
    pub fn add_entry(&mut self, entry: HashChainEntry) -> Result<(), String> {
        // Verify entry integrity
        if !entry.verify_hash() {
            return Err("Entry hash verification failed".to_string());
        }

        // Verify chain link
        if !self.entries.is_empty() {
            if !self.entries.last().unwrap().verify_link(&entry) {
                return Err("Chain link verification failed".to_string());
            }
        } else if entry.previous_hash != "0".repeat(64) {
            return Err("Genesis entry must have zero previous_hash".to_string());
        }

        self.head_hash = entry.entry_hash.clone();
        self.entries.push(entry);
        self.integrity_verified = false; // Reset until re-verified

        Ok(())
    }

    /// Verify complete chain integrity.
    pub fn verify(&mut self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();

        // Check entries
        for (i, entry) in self.entries.iter().enumerate() {
            // Verify entry hash
            if !entry.verify_hash() {
                issues.push(format!(
                    "Entry {} hash mismatch: {} != computed",
                    entry.sequence_number, entry.entry_hash
                ));
            }

            // Verify chain link
            if i > 0 {
                let previous = &self.entries[i - 1];
                if entry.previous_hash != previous.entry_hash {
                    issues.push(format!(
                        "Entry {} chain break: expected {}, got {}",
                        entry.sequence_number, previous.entry_hash, entry.previous_hash
                    ));
                }
            } else {
                // Genesis entry check
                if entry.previous_hash != "0".repeat(64) {
                    issues.push(format!(
                        "Genesis entry {} should have zero previous_hash",
                        entry.sequence_number
                    ));
                }
            }
        }

        if issues.is_empty() {
            self.integrity_verified = true;
            self.verified_at = Some(Utc::now());
            self.merkle_root = self.compute_merkle_root();
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Compute Merkle tree root hash.
    fn compute_merkle_root(&self) -> String {
        use sha2::{Digest, Sha256};

        if self.entries.is_empty() {
            return "0".repeat(64);
        }

        let mut hashes: Vec<String> = self.entries.iter().map(|e| e.entry_hash.clone()).collect();

        // Build Merkle tree bottom-up
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

        hashes.pop().unwrap_or_else(|| "0".repeat(64))
    }

    /// Get all entries.
    pub fn get_entries(&self) -> &[HashChainEntry] {
        &self.entries
    }

    /// Get entry by sequence number.
    pub fn get_entry(&self, sequence: u64) -> Option<&HashChainEntry> {
        self.entries.iter().find(|e| e.sequence_number == sequence)
    }

    /// Get entry by event ID.
    pub fn get_entry_by_id(&self, event_id: Uuid) -> Option<&HashChainEntry> {
        self.entries.iter().find(|e| e.event_id == event_id)
    }

    /// Export chain as JSON.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export chain as CSV.
    pub fn export_csv(&self) -> String {
        let mut csv = String::from(
            "sequence_number,event_id,timestamp,event_type,entry_hash,previous_hash\n",
        );

        for entry in &self.entries {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                entry.sequence_number,
                entry.event_id,
                entry.timestamp,
                entry.event_type,
                entry.entry_hash,
                entry.previous_hash
            ));
        }

        csv
    }

    /// Get chain statistics.
    pub fn statistics(&self) -> ChainStatistics {
        ChainStatistics {
            total_entries: self.entries.len() as u64,
            entry_count_by_type: self.count_by_type(),
            entry_count_by_category: self.count_by_category(),
            integrity_verified: self.integrity_verified,
            head_hash: self.head_hash.clone(),
            merkle_root: self.merkle_root.clone(),
        }
    }

    /// Count entries by type.
    fn count_by_type(&self) -> std::collections::HashMap<String, u64> {
        use std::collections::HashMap;

        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.event_type.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Count entries by category.
    fn count_by_category(&self) -> std::collections::HashMap<String, u64> {
        use std::collections::HashMap;

        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.event_category.clone()).or_insert(0) += 1;
        }
        counts
    }
}

/// Chain statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStatistics {
    pub total_entries: u64,
    pub entry_count_by_type: std::collections::HashMap<String, u64>,
    pub entry_count_by_category: std::collections::HashMap<String, u64>,
    pub integrity_verified: bool,
    pub head_hash: String,
    pub merkle_root: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_chain_entry_creation() {
        let entry = HashChainEntry::from_event(
            1,
            Uuid::new_v4(),
            Utc::now().to_rfc3339(),
            "test_event".to_string(),
            "test_category".to_string(),
            "0".repeat(64),
            serde_json::json!({"key": "value"}),
        );

        assert_eq!(entry.sequence_number, 1);
        assert_eq!(entry.entry_hash.len(), 64); // SHA256 hex = 64 chars
    }

    #[test]
    fn test_hash_chain_verification() {
        let mut chain = HashChain::new(Uuid::new_v4());

        let entry1 = HashChainEntry::from_event(
            1,
            Uuid::new_v4(),
            Utc::now().to_rfc3339(),
            "event1".to_string(),
            "category".to_string(),
            "0".repeat(64),
            serde_json::json!({"data": "test"}),
        );

        assert!(chain.add_entry(entry1).is_ok());

        let entry2 = HashChainEntry::from_event(
            2,
            Uuid::new_v4(),
            Utc::now().to_rfc3339(),
            "event2".to_string(),
            "category".to_string(),
            chain.head_hash.clone(),
            serde_json::json!({"data": "test2"}),
        );

        assert!(chain.add_entry(entry2).is_ok());

        let result = chain.verify();
        assert!(result.is_ok());
        assert!(chain.integrity_verified);
    }

    #[test]
    fn test_chain_tamper_detection() {
        let mut chain = HashChain::new(Uuid::new_v4());

        let mut entry = HashChainEntry::from_event(
            1,
            Uuid::new_v4(),
            Utc::now().to_rfc3339(),
            "event1".to_string(),
            "category".to_string(),
            "0".repeat(64),
            serde_json::json!({"data": "original"}),
        );

        chain.add_entry(entry.clone()).expect("test setup failed");

        // Tamper with entry
        entry.payload = serde_json::json!({"data": "modified"});
        assert!(!entry.verify_hash());
    }

    #[test]
    fn test_merkle_root_computation() {
        let mut chain = HashChain::new(Uuid::new_v4());

        for i in 0..4 {
            let entry = HashChainEntry::from_event(
                (i + 1) as u64,
                Uuid::new_v4(),
                Utc::now().to_rfc3339(),
                format!("event{}", i),
                "category".to_string(),
                if i == 0 {
                    "0".repeat(64)
                } else {
                    chain.head_hash.clone()
                },
                serde_json::json!({"index": i}),
            );

            chain.add_entry(entry).expect("test setup failed");
        }

        let merkle = chain.compute_merkle_root();
        assert_eq!(merkle.len(), 64);
    }
}
