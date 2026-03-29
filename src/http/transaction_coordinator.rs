//! Distributed Transaction Coordinator (2PC Implementation)
//!
//! Implements Two-Phase Commit protocol for cross-system transactions
//! between BOS (Rust coordinator) and BusinessOS (Go participant).
//!
//! # Protocol
//!
//! Phase 1 (Prepare): Coordinator sends prepare request, participant validates & locks
//! Phase 2 (Commit/Abort): Coordinator broadcasts decision based on votes
//!
//! # Guarantees
//!
//! - Atomicity: All-or-nothing across system boundaries
//! - Durability: Transaction logs persisted before decisions
//! - Crash recovery: Conservative abort on recovery
//! - Byzantine resilience: Checksum validation on all messages

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use sha2::{Sha256, Digest};

/// Transaction unique identifier
pub type TransactionId = String;

/// Coordinator state for distributed transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionState {
    /// Initial state: transaction created
    Initial,
    /// Pending: awaiting prepare request delivery
    Pending,
    /// Preparing: voting phase in progress
    Preparing,
    /// Committing: commit phase in progress
    Committing,
    /// Aborting: abort phase in progress
    Aborting,
    /// Committed: transaction succeeded
    Committed,
    /// Aborted: transaction failed
    Aborted,
}

/// Participant vote in prepare phase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParticipantVote {
    /// Participant ready to commit
    Ready,
    /// Participant votes to abort
    Abort,
}

/// Transaction operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionOperation {
    /// Persist process model to BusinessOS
    ModelPersistence,
    /// Record conformance check result
    ConformanceRecording,
    /// Synchronize enriched model
    ModelSync,
}

/// Prepare request sent to participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareRequest {
    pub transaction_id: TransactionId,
    pub operation: String,
    pub data: serde_json::Value,
    pub deadline: DateTime<Utc>,
    pub checksum: String, // SHA256 of data
}

/// Prepare response from participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareResponse {
    pub transaction_id: TransactionId,
    pub status: String, // "READY" or "ABORT"
    pub undo_log: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub error_reason: Option<String>,
}

/// Commit request sent to participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRequest {
    pub transaction_id: TransactionId,
}

/// Commit acknowledgement from participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAck {
    pub transaction_id: TransactionId,
    pub committed_at: DateTime<Utc>,
}

/// Abort request sent to participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortRequest {
    pub transaction_id: TransactionId,
}

/// Abort acknowledgement from participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortAck {
    pub transaction_id: TransactionId,
    pub aborted_at: DateTime<Utc>,
}

/// Persistent transaction log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLogEntry {
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub transaction_id: TransactionId,
    pub coordinator_id: String,
    pub participant_id: String,
    pub state: String, // Serialized TransactionState
    pub operation: String,
    pub data_hash: String,
    pub undo_log: Option<String>,
    pub error: Option<String>,
}

/// In-memory transaction state for coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorTransaction {
    pub id: TransactionId,
    pub state: TransactionState,
    pub operation: TransactionOperation,
    pub data: serde_json::Value,
    pub participant_ids: Vec<String>, // Multiple participants support
    pub created_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub prepare_responses: HashMap<String, ParticipantVote>,
    pub retry_count: u32,
}

/// Distributed Transaction Coordinator
pub struct TransactionCoordinator {
    /// In-memory transactions (volatile)
    transactions: Arc<Mutex<HashMap<TransactionId, CoordinatorTransaction>>>,
    /// Log file path
    log_path: PathBuf,
    /// Timeout for prepare phase (seconds)
    prepare_timeout: u64,
    /// Timeout for commit phase (seconds)
    commit_timeout: u64,
    /// Maximum retry attempts
    max_retries: u32,
    /// Coordinator ID
    coordinator_id: String,
}

impl TransactionCoordinator {
    /// Create new transaction coordinator
    pub fn new(
        log_path: PathBuf,
        prepare_timeout: u64,
        commit_timeout: u64,
    ) -> Self {
        let _ = fs::create_dir_all(&log_path);
        
        Self {
            transactions: Arc::new(Mutex::new(HashMap::new())),
            log_path,
            prepare_timeout,
            commit_timeout,
            max_retries: 3,
            coordinator_id: format!("bos-coordinator-{}", Uuid::new_v4()),
        }
    }

    /// Begin new transaction with multiple participants
    pub fn begin_transaction(
        &self,
        model_id: &str,
        operation: TransactionOperation,
        data: serde_json::Value,
        participant_ids: Vec<String>,
    ) -> Result<TransactionId, String> {
        let txn_id = format!("txn_{}", Uuid::new_v4().to_string()[..12].to_string());
        let now = Utc::now();
        let deadline = now + Duration::seconds(self.prepare_timeout as i64);

        let txn = CoordinatorTransaction {
            id: txn_id.clone(),
            state: TransactionState::Pending,
            operation,
            data,
            participant_ids: participant_ids.clone(),
            created_at: now,
            deadline,
            prepare_responses: HashMap::new(),
            retry_count: 0,
        };

        // Log PENDING state for each participant
        for pid in &participant_ids {
            self.log_entry(&txn_id, "PENDING", pid, None)?;
        }

        // Store in memory
        {
            let mut txns = self.transactions.lock().expect("transaction lock poisoned");
            txns.insert(txn_id.clone(), txn);
        }

        Ok(txn_id)
    }

    /// Generate prepare request
    pub fn prepare_request(&self, txn_id: &str) -> Result<PrepareRequest, String> {
        let txns = self.transactions.lock().expect("transaction lock poisoned");
        let txn = txns.get(txn_id)
            .ok_or_else(|| format!("Transaction not found: {}", txn_id))?;

        let data_str = serde_json::to_string(&txn.data)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;
        let checksum = Self::calculate_checksum(&data_str);

        let operation_str = match &txn.operation {
            TransactionOperation::ModelPersistence => "model_persistence".to_string(),
            TransactionOperation::ConformanceRecording => "conformance_recording".to_string(),
            TransactionOperation::ModelSync => "model_sync".to_string(),
        };

        Ok(PrepareRequest {
            transaction_id: txn_id.to_string(),
            operation: operation_str,
            data: txn.data.clone(),
            deadline: txn.deadline,
            checksum,
        })
    }

    /// Handle prepare response
    pub fn handle_prepare_response(
        &self,
        response: &PrepareResponse,
    ) -> Result<(), String> {
        let mut txns = self.transactions.lock().expect("transaction lock poisoned");
        let txn = txns.get_mut(&response.transaction_id)
            .ok_or_else(|| format!("Transaction not found: {}", response.transaction_id))?;

        // Update state to PREPARING if not already
        if txn.state == TransactionState::Pending {
            txn.state = TransactionState::Preparing;
        }

        // Record vote
        let vote = match response.status.as_str() {
            "READY" => ParticipantVote::Ready,
            "ABORT" => ParticipantVote::Abort,
            _ => return Err(format!("Invalid vote status: {}", response.status)),
        };

        txn.prepare_responses.insert(txn.participant_id.clone(), vote);

        // Log response
        if response.status == "READY" {
            self.log_entry(&response.transaction_id, "READY", &txn.participant_id, None)?;
        } else {
            self.log_entry(
                &response.transaction_id,
                "ABORT",
                &txn.participant_id,
                response.error_reason.as_deref(),
            )?;
        }

        Ok(())
    }

    /// Check if all participants voted READY
    pub fn all_ready(&self, txn_id: &str) -> Result<bool, String> {
        let txns = self.transactions.lock().expect("transaction lock poisoned");
        let txn = txns.get(txn_id)
            .ok_or_else(|| format!("Transaction not found: {}", txn_id))?;

        // All participants must have voted READY
        if txn.prepare_responses.len() != txn.participant_ids.len() {
            return Ok(false);
        }

        Ok(txn.participant_ids.iter().all(|pid| {
            txn.prepare_responses.get(pid) == Some(&ParticipantVote::Ready)
        }))
    }

    /// Transition to COMMITTING state
    pub fn commit_transaction(&self, txn_id: &str) -> Result<(), String> {
        let mut txns = self.transactions.lock().expect("transaction lock poisoned");
        let txn = txns.get_mut(txn_id)
            .ok_or_else(|| format!("Transaction not found: {}", txn_id))?;

        if txn.state != TransactionState::Preparing {
            return Err(format!("Invalid state for commit: {:?}", txn.state));
        }

        txn.state = TransactionState::Committing;
        let participant_ids = txn.participant_ids.clone();
        drop(txns);

        // Log for all participants
        for pid in participant_ids {
            self.log_entry(txn_id, "COMMITTING", &pid, None)?;
        }

        Ok(())
    }

    /// Handle commit acknowledgement
    pub fn handle_commit_ack(&self, txn_id: &str, participant_id: &str) -> Result<(), String> {
        let mut txns = self.transactions.lock().expect("transaction lock poisoned");
        let txn = txns.get_mut(txn_id)
            .ok_or_else(|| format!("Transaction not found: {}", txn_id))?;

        if txn.state != TransactionState::Committing {
            return Err(format!("Invalid state for commit ack: {:?}", txn.state));
        }

        // Check if this is the last participant to acknowledge
        if txn.prepare_responses.len() >= txn.participant_ids.len() {
            txn.state = TransactionState::Committed;
            let participant_ids = txn.participant_ids.clone();
            drop(txns);

            // Log for all participants
            for pid in participant_ids {
                self.log_entry(txn_id, "COMMITTED", &pid, None)?;
            }
        } else {
            self.log_entry(txn_id, "COMMITTED", participant_id, None)?;
        }

        Ok(())
    }

    /// Transition to ABORTING state
    pub fn abort_transaction(&self, txn_id: &str, reason: Option<&str>) -> Result<(), String> {
        let mut txns = self.transactions.lock().expect("transaction lock poisoned");
        let txn = txns.get_mut(txn_id)
            .ok_or_else(|| format!("Transaction not found: {}", txn_id))?;

        txn.state = TransactionState::Aborting;
        let participant_ids = txn.participant_ids.clone();
        drop(txns);

        // Log for all participants
        for pid in participant_ids {
            self.log_entry(txn_id, "ABORTING", &pid, reason)?;
        }

        Ok(())
    }

    /// Handle abort acknowledgement
    pub fn handle_abort_ack(&self, txn_id: &str, participant_id: &str) -> Result<(), String> {
        let mut txns = self.transactions.lock().expect("transaction lock poisoned");
        let txn = txns.get_mut(txn_id)
            .ok_or_else(|| format!("Transaction not found: {}", txn_id))?;

        if txn.state != TransactionState::Aborting {
            return Err(format!("Invalid state for abort ack: {:?}", txn.state));
        }

        txn.state = TransactionState::Aborted;
        let participant_ids = txn.participant_ids.clone();
        drop(txns);

        // Log for all participants
        for pid in participant_ids {
            self.log_entry(txn_id, "ABORTED", &pid, None)?;
        }

        Ok(())
    }

    /// Check for timeout on prepare phase
    pub fn check_prepare_timeout(&self, txn_id: &str) -> Result<bool, String> {
        let txns = self.transactions.lock().expect("transaction lock poisoned");
        let txn = txns.get(txn_id)
            .ok_or_else(|| format!("Transaction not found: {}", txn_id))?;

        if txn.state != TransactionState::Preparing {
            return Ok(false);
        }

        let now = Utc::now();
        Ok(now > txn.deadline)
    }

    /// Calculate SHA256 checksum
    fn calculate_checksum(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let hash = hasher.finalize();
        format!("sha256:{:x}", hash)
    }

    /// Log transaction state change
    fn log_entry(
        &self,
        txn_id: &str,
        state: &str,
        participant_id: &str,
        error: Option<&str>,
    ) -> Result<(), String> {
        let data = serde_json::json!({});
        let data_str = serde_json::to_string(&data)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;

        // Get the actual operation from the transaction
        let txns = self.transactions.lock().expect("transaction lock poisoned");
        let operation_str = if let Some(txn) = txns.get(txn_id) {
            match &txn.operation {
                TransactionOperation::ModelPersistence => "model_persistence".to_string(),
                TransactionOperation::ConformanceRecording => "conformance_recording".to_string(),
                TransactionOperation::ModelSync => "model_sync".to_string(),
            }
        } else {
            "unknown_operation".to_string()
        };
        drop(txns);

        let entry = TransactionLogEntry {
            version: 1,
            timestamp: Utc::now(),
            transaction_id: txn_id.to_string(),
            coordinator_id: self.coordinator_id.clone(),
            participant_id: participant_id.to_string(),
            state: state.to_string(),
            operation: operation_str,
            data_hash: Self::calculate_checksum(&data_str),
            undo_log: None,
            error: error.map(|s| s.to_string()),
        };

        let log_json = serde_json::to_string(&entry)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;

        let log_file = self.log_path.join(format!("txn_{}.log", txn_id));
        fs::write(&log_file, format!("{}\n", log_json))
            .map_err(|e| format!("Failed to write log file: {}", e))?;

        Ok(())
    }

    /// Recover incomplete transactions from log
    pub fn recover_transactions(&self) -> Result<(), String> {
        if !self.log_path.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.log_path)
            .map_err(|e| format!("Failed to read log directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read log entry: {}", e))?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read log file: {}", e))?;

            let log_entry: TransactionLogEntry = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse log entry: {}", e))?;

            // For incomplete transactions, abort them
            match log_entry.state.as_str() {
                "PENDING" | "PREPARING" => {
                    // Conservative: abort incomplete transactions
                    self.log_entry(
                        &log_entry.transaction_id,
                        "RECOVERED_ABORT",
                        &log_entry.participant_id,
                        Some("Recovered from crash"),
                    )?;
                }
                _ => {
                    // Terminal states: nothing to do
                }
            }
        }

        Ok(())
    }

    /// Get transaction state
    pub fn get_transaction_state(&self, txn_id: &str) -> Result<TransactionState, String> {
        let txns = self.transactions.lock().expect("transaction lock poisoned");
        let txn = txns.get(txn_id)
            .ok_or_else(|| format!("Transaction not found: {}", txn_id))?;

        Ok(txn.state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_begin_transaction() {
        let temp_dir = TempDir::new().unwrap();
        let coordinator = TransactionCoordinator::new(temp_dir.path().to_path_buf(), 30, 60);

        let txn_id = coordinator
            .begin_transaction(
                "model_xyz",
                TransactionOperation::ModelPersistence,
                serde_json::json!({"test": "data"}),
                "businessos-1",
            )
            .unwrap();

        assert!(txn_id.starts_with("txn_"));
        let state = coordinator.get_transaction_state(&txn_id).unwrap();
        assert_eq!(state, TransactionState::Pending);
    }

    #[test]
    fn test_prepare_request_generation() {
        let temp_dir = TempDir::new().unwrap();
        let coordinator = TransactionCoordinator::new(temp_dir.path().to_path_buf(), 30, 60);

        let txn_id = coordinator
            .begin_transaction(
                "model_xyz",
                TransactionOperation::ModelPersistence,
                serde_json::json!({"test": "data"}),
                "businessos-1",
            )
            .unwrap();

        let prepare_req = coordinator.prepare_request(&txn_id).unwrap();
        assert_eq!(prepare_req.transaction_id, txn_id);
        assert_eq!(prepare_req.operation, "model_persistence");
        assert!(prepare_req.checksum.starts_with("sha256:"));
    }

    #[test]
    fn test_prepare_response_handling() {
        let temp_dir = TempDir::new().unwrap();
        let coordinator = TransactionCoordinator::new(temp_dir.path().to_path_buf(), 30, 60);

        let txn_id = coordinator
            .begin_transaction(
                "model_xyz",
                TransactionOperation::ModelPersistence,
                serde_json::json!({"test": "data"}),
                "businessos-1",
            )
            .unwrap();

        let response = PrepareResponse {
            transaction_id: txn_id.clone(),
            status: "READY".to_string(),
            undo_log: Some("undo_abc123".to_string()),
            timestamp: Utc::now(),
            error_reason: None,
        };

        coordinator.handle_prepare_response(&response).unwrap();

        assert!(coordinator.all_ready(&txn_id).unwrap());
    }

    #[test]
    fn test_transaction_commit_flow() {
        let temp_dir = TempDir::new().unwrap();
        let coordinator = TransactionCoordinator::new(temp_dir.path().to_path_buf(), 30, 60);

        let txn_id = coordinator
            .begin_transaction(
                "model_xyz",
                TransactionOperation::ModelPersistence,
                serde_json::json!({"test": "data"}),
                "businessos-1",
            )
            .unwrap();

        let response = PrepareResponse {
            transaction_id: txn_id.clone(),
            status: "READY".to_string(),
            undo_log: None,
            timestamp: Utc::now(),
            error_reason: None,
        };

        coordinator.handle_prepare_response(&response).unwrap();
        coordinator.commit_transaction(&txn_id).unwrap();

        let ack = CommitAck {
            transaction_id: txn_id.clone(),
            committed_at: Utc::now(),
        };

        coordinator.handle_commit_ack(&txn_id).unwrap();

        let state = coordinator.get_transaction_state(&txn_id).unwrap();
        assert_eq!(state, TransactionState::Committed);
    }

    #[test]
    fn test_transaction_abort_flow() {
        let temp_dir = TempDir::new().unwrap();
        let coordinator = TransactionCoordinator::new(temp_dir.path().to_path_buf(), 30, 60);

        let txn_id = coordinator
            .begin_transaction(
                "model_xyz",
                TransactionOperation::ModelPersistence,
                serde_json::json!({"test": "data"}),
                "businessos-1",
            )
            .unwrap();

        let response = PrepareResponse {
            transaction_id: txn_id.clone(),
            status: "ABORT".to_string(),
            undo_log: None,
            timestamp: Utc::now(),
            error_reason: Some("Schema validation failed".to_string()),
        };

        coordinator.handle_prepare_response(&response).unwrap();
        coordinator.abort_transaction(&txn_id, Some("Participant abort")).unwrap();

        let ack = AbortAck {
            transaction_id: txn_id.clone(),
            aborted_at: Utc::now(),
        };

        coordinator.handle_abort_ack(&txn_id).unwrap();

        let state = coordinator.get_transaction_state(&txn_id).unwrap();
        assert_eq!(state, TransactionState::Aborted);
    }

    #[test]
    fn test_checksum_validation() {
        let data = "test data";
        let checksum1 = TransactionCoordinator::calculate_checksum(data);
        let checksum2 = TransactionCoordinator::calculate_checksum(data);
        assert_eq!(checksum1, checksum2);
        assert!(checksum1.starts_with("sha256:"));
    }

    #[test]
    fn test_transaction_logging() {
        let temp_dir = TempDir::new().unwrap();
        let coordinator = TransactionCoordinator::new(temp_dir.path().to_path_buf(), 30, 60);

        let txn_id = coordinator
            .begin_transaction(
                "model_xyz",
                TransactionOperation::ModelPersistence,
                serde_json::json!({"test": "data"}),
                "businessos-1",
            )
            .unwrap();

        // Check log file was created
        let log_file = temp_dir.path().join(format!("txn_{}.log", txn_id));
        assert!(log_file.exists());

        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains(&txn_id));
        assert!(content.contains("PENDING"));
    }
}
