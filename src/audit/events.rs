//! Audit event type definitions.
//!
//! All events follow Signal Theory encoding: S=(M,G,T,F,W)
//! where Mode=data, Genre=audit, Type=inform, Format=json, Structure=event

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Complete audit event for BOS ↔ BusinessOS operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub event_id: Uuid,

    /// Event type (e.g., "model_discovered", "conformance_checked")
    pub event_type: String,

    /// Event category (ProcessMining, Compliance, Integration, Security)
    pub event_category: String,

    /// ISO8601 timestamp
    pub timestamp: String,

    /// Severity level
    pub severity: String,

    /// Actor context
    pub actor: ActorContext,

    /// Resource context (optional)
    pub resource: Option<ResourceContext>,

    /// Action details
    pub action: ActionDetails,

    /// Compliance metadata
    pub compliance: ComplianceMetadata,

    /// Hash chain metadata
    pub chain: ChainMetadata,
}

/// Actor context (who performed the operation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// User UUID
    pub user_id: Uuid,

    /// User email (if available)
    pub email: Option<String>,

    /// User role
    pub role: String,

    /// IP address
    pub ip_address: Option<String>,

    /// Session ID
    pub session_id: Option<Uuid>,
}

/// Resource context (what was affected).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContext {
    /// Resource type (process_model, log, statistic, etc.)
    pub resource_type: String,

    /// Resource UUID
    pub resource_id: Uuid,

    /// Resource name (if available)
    pub name: Option<String>,

    /// Workspace UUID
    pub workspace_id: Option<Uuid>,
}

/// Action details (what happened).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDetails {
    /// Operation name
    pub operation: String,

    /// Operation-specific fields
    pub details: serde_json::Value,
}

/// Compliance metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMetadata {
    /// GDPR classification
    pub gdpr_classification: Option<String>,

    /// Count of data subjects affected
    pub data_subjects_affected: Option<u32>,

    /// Whether PII is detected
    pub pii_detected: bool,

    /// Legal hold flag
    pub legal_hold: bool,
}

/// Hash chain metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetadata {
    /// Sequence number in the chain
    pub sequence_number: u64,

    /// Hash of this entry
    pub entry_hash: String,

    /// Hash of previous entry
    pub previous_hash: String,

    /// Merkle tree hash for period
    pub merkle_tree_hash: Option<String>,
}

/// Supported event types.
#[derive(Debug, Clone)]
pub enum AuditEventType {
    ModelDiscovered,
    ConformanceChecked,
    StatisticsComputed,
    ModelVariantDiscovered,
    AccessGranted,
    AccessRevoked,
    ExportRequested,
    DataDeletion,
    ConfigurationChanged,
    ApiKeyCreated,
    ApiCallReceived,
    ApiQuotaExceeded,
    AuthenticationFailure,
    PrivilegeEscalationAttempt,
    SuspiciousActivityDetected,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::ModelDiscovered => write!(f, "model_discovered"),
            AuditEventType::ConformanceChecked => write!(f, "conformance_checked"),
            AuditEventType::StatisticsComputed => write!(f, "statistics_computed"),
            AuditEventType::ModelVariantDiscovered => write!(f, "model_variant_discovered"),
            AuditEventType::AccessGranted => write!(f, "access_granted"),
            AuditEventType::AccessRevoked => write!(f, "access_revoked"),
            AuditEventType::ExportRequested => write!(f, "export_requested"),
            AuditEventType::DataDeletion => write!(f, "data_deletion"),
            AuditEventType::ConfigurationChanged => write!(f, "configuration_changed"),
            AuditEventType::ApiKeyCreated => write!(f, "api_key_created"),
            AuditEventType::ApiCallReceived => write!(f, "api_call_received"),
            AuditEventType::ApiQuotaExceeded => write!(f, "api_quota_exceeded"),
            AuditEventType::AuthenticationFailure => write!(f, "authentication_failure"),
            AuditEventType::PrivilegeEscalationAttempt => {
                write!(f, "privilege_escalation_attempt")
            }
            AuditEventType::SuspiciousActivityDetected => write!(f, "suspicious_activity_detected"),
        }
    }
}

/// Supported event categories.
#[derive(Debug, Clone)]
pub enum AuditCategory {
    ProcessMining,
    Compliance,
    Integration,
    Security,
}

impl std::fmt::Display for AuditCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditCategory::ProcessMining => write!(f, "ProcessMining"),
            AuditCategory::Compliance => write!(f, "Compliance"),
            AuditCategory::Integration => write!(f, "Integration"),
            AuditCategory::Security => write!(f, "Security"),
        }
    }
}

impl AuditEvent {
    /// Create a "model_discovered" event.
    pub fn model_discovered(
        user_id: Uuid,
        log_source: String,
        algorithm: String,
        result_hash: String,
        activities_count: u32,
        duration_ms: u64,
    ) -> Self {
        let details = serde_json::json!({
            "log_source": log_source,
            "algorithm": algorithm,
            "result_hash": result_hash,
            "activities_count": activities_count,
            "duration_ms": duration_ms,
        });

        Self {
            event_id: Uuid::new_v4(),
            event_type: AuditEventType::ModelDiscovered.to_string(),
            event_category: AuditCategory::ProcessMining.to_string(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            severity: "info".to_string(),
            actor: ActorContext {
                user_id,
                email: None,
                role: "analyst".to_string(),
                ip_address: None,
                session_id: None,
            },
            resource: None,
            action: ActionDetails {
                operation: "model_discovered".to_string(),
                details,
            },
            compliance: ComplianceMetadata {
                gdpr_classification: Some("processing_activity".to_string()),
                data_subjects_affected: None,
                pii_detected: false,
                legal_hold: false,
            },
            chain: ChainMetadata {
                sequence_number: 0,
                entry_hash: String::new(),
                previous_hash: String::new(),
                merkle_tree_hash: None,
            },
        }
    }

    /// Create a "conformance_checked" event.
    pub fn conformance_checked(
        user_id: Uuid,
        model_id: Uuid,
        log_id: Uuid,
        fitness: f64,
        precision: f64,
        generalization: f64,
        log_entries_tested: u32,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: AuditEventType::ConformanceChecked.to_string(),
            event_category: AuditCategory::ProcessMining.to_string(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            severity: "info".to_string(),
            actor: ActorContext {
                user_id,
                email: None,
                role: "analyst".to_string(),
                ip_address: None,
                session_id: None,
            },
            resource: Some(ResourceContext {
                resource_type: "process_model".to_string(),
                resource_id: model_id,
                name: None,
                workspace_id: None,
            }),
            action: ActionDetails {
                operation: "conformance_checked".to_string(),
                details: serde_json::json!({
                    "model_id": model_id,
                    "log_id": log_id,
                    "fitness": fitness,
                    "precision": precision,
                    "generalization": generalization,
                    "log_entries_tested": log_entries_tested,
                }),
            },
            compliance: ComplianceMetadata {
                gdpr_classification: Some("processing_activity".to_string()),
                data_subjects_affected: Some(log_entries_tested),
                pii_detected: false,
                legal_hold: false,
            },
            chain: ChainMetadata {
                sequence_number: 0,
                entry_hash: String::new(),
                previous_hash: String::new(),
                merkle_tree_hash: None,
            },
        }
    }

    /// Create a "statistics_computed" event.
    pub fn statistics_computed(
        user_id: Uuid,
        log_id: Uuid,
        statistic_type: String,
        result_hash: String,
        sample_size: u32,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: AuditEventType::StatisticsComputed.to_string(),
            event_category: AuditCategory::ProcessMining.to_string(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            severity: "info".to_string(),
            actor: ActorContext {
                user_id,
                email: None,
                role: "analyst".to_string(),
                ip_address: None,
                session_id: None,
            },
            resource: Some(ResourceContext {
                resource_type: "log".to_string(),
                resource_id: log_id,
                name: None,
                workspace_id: None,
            }),
            action: ActionDetails {
                operation: "statistics_computed".to_string(),
                details: serde_json::json!({
                    "statistic_type": statistic_type,
                    "result_hash": result_hash,
                    "sample_size": sample_size,
                }),
            },
            compliance: ComplianceMetadata {
                gdpr_classification: Some("analytics".to_string()),
                data_subjects_affected: Some(sample_size),
                pii_detected: false,
                legal_hold: false,
            },
            chain: ChainMetadata {
                sequence_number: 0,
                entry_hash: String::new(),
                previous_hash: String::new(),
                merkle_tree_hash: None,
            },
        }
    }

    /// Create an "access_granted" event.
    pub fn access_granted(
        user_id: Uuid,
        resource_id: Uuid,
        resource_type: String,
        permission: String,
        granted_by: Uuid,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: AuditEventType::AccessGranted.to_string(),
            event_category: AuditCategory::Compliance.to_string(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            severity: "info".to_string(),
            actor: ActorContext {
                user_id: granted_by,
                email: None,
                role: "admin".to_string(),
                ip_address: None,
                session_id: None,
            },
            resource: Some(ResourceContext {
                resource_type,
                resource_id,
                name: None,
                workspace_id: None,
            }),
            action: ActionDetails {
                operation: "access_granted".to_string(),
                details: serde_json::json!({
                    "target_user_id": user_id,
                    "permission": permission,
                    "granted_by": granted_by,
                }),
            },
            compliance: ComplianceMetadata {
                gdpr_classification: Some("access_control".to_string()),
                data_subjects_affected: Some(1),
                pii_detected: false,
                legal_hold: false,
            },
            chain: ChainMetadata {
                sequence_number: 0,
                entry_hash: String::new(),
                previous_hash: String::new(),
                merkle_tree_hash: None,
            },
        }
    }

    /// Create an "access_revoked" event.
    pub fn access_revoked(
        user_id: Uuid,
        resource_id: Uuid,
        revoked_by: Uuid,
        reason: String,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: AuditEventType::AccessRevoked.to_string(),
            event_category: AuditCategory::Compliance.to_string(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            severity: "warning".to_string(),
            actor: ActorContext {
                user_id: revoked_by,
                email: None,
                role: "admin".to_string(),
                ip_address: None,
                session_id: None,
            },
            resource: Some(ResourceContext {
                resource_type: "resource".to_string(),
                resource_id,
                name: None,
                workspace_id: None,
            }),
            action: ActionDetails {
                operation: "access_revoked".to_string(),
                details: serde_json::json!({
                    "target_user_id": user_id,
                    "revoked_by": revoked_by,
                    "reason": reason,
                }),
            },
            compliance: ComplianceMetadata {
                gdpr_classification: Some("access_control".to_string()),
                data_subjects_affected: Some(1),
                pii_detected: false,
                legal_hold: false,
            },
            chain: ChainMetadata {
                sequence_number: 0,
                entry_hash: String::new(),
                previous_hash: String::new(),
                merkle_tree_hash: None,
            },
        }
    }

    /// Create a "data_deletion" event (GDPR right to be forgotten).
    pub fn data_deletion(
        user_id: Uuid,
        resource_id: Uuid,
        deletion_reason: String,
        previous_hash: String,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: AuditEventType::DataDeletion.to_string(),
            event_category: AuditCategory::Compliance.to_string(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            severity: "critical".to_string(),
            actor: ActorContext {
                user_id,
                email: None,
                role: "admin".to_string(),
                ip_address: None,
                session_id: None,
            },
            resource: Some(ResourceContext {
                resource_type: "resource".to_string(),
                resource_id,
                name: None,
                workspace_id: None,
            }),
            action: ActionDetails {
                operation: "data_deletion".to_string(),
                details: serde_json::json!({
                    "deletion_reason": deletion_reason,
                    "previous_hash": previous_hash,
                }),
            },
            compliance: ComplianceMetadata {
                gdpr_classification: Some("right_to_be_forgotten".to_string()),
                data_subjects_affected: Some(1),
                pii_detected: true,
                legal_hold: false,
            },
            chain: ChainMetadata {
                sequence_number: 0,
                entry_hash: String::new(),
                previous_hash: String::new(),
                merkle_tree_hash: None,
            },
        }
    }

    /// Create an "authentication_failure" event.
    pub fn authentication_failure(
        username_hash: String,
        ip_address: String,
        failure_reason: String,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: AuditEventType::AuthenticationFailure.to_string(),
            event_category: AuditCategory::Security.to_string(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            severity: "warning".to_string(),
            actor: ActorContext {
                user_id: Uuid::nil(),
                email: None,
                role: "unknown".to_string(),
                ip_address: Some(ip_address.clone()),
                session_id: None,
            },
            resource: None,
            action: ActionDetails {
                operation: "authentication_failure".to_string(),
                details: serde_json::json!({
                    "username_hash": username_hash,
                    "ip_address": ip_address,
                    "failure_reason": failure_reason,
                }),
            },
            compliance: ComplianceMetadata {
                gdpr_classification: Some("security_event".to_string()),
                data_subjects_affected: None,
                pii_detected: false,
                legal_hold: false,
            },
            chain: ChainMetadata {
                sequence_number: 0,
                entry_hash: String::new(),
                previous_hash: String::new(),
                merkle_tree_hash: None,
            },
        }
    }

    /// Create a "privilege_escalation_attempt" event.
    pub fn privilege_escalation_attempt(
        user_id: Uuid,
        attempted_role: String,
        action_taken: String,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: AuditEventType::PrivilegeEscalationAttempt.to_string(),
            event_category: AuditCategory::Security.to_string(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            severity: "critical".to_string(),
            actor: ActorContext {
                user_id,
                email: None,
                role: "user".to_string(),
                ip_address: None,
                session_id: None,
            },
            resource: None,
            action: ActionDetails {
                operation: "privilege_escalation_attempt".to_string(),
                details: serde_json::json!({
                    "attempted_role": attempted_role,
                    "action_taken": action_taken,
                }),
            },
            compliance: ComplianceMetadata {
                gdpr_classification: Some("security_event".to_string()),
                data_subjects_affected: None,
                pii_detected: false,
                legal_hold: false,
            },
            chain: ChainMetadata {
                sequence_number: 0,
                entry_hash: String::new(),
                previous_hash: String::new(),
                merkle_tree_hash: None,
            },
        }
    }

    /// Create a "suspicious_activity_detected" event.
    pub fn suspicious_activity_detected(
        user_id: Uuid,
        activity_type: String,
        confidence_score: f64,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: AuditEventType::SuspiciousActivityDetected.to_string(),
            event_category: AuditCategory::Security.to_string(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            severity: "warning".to_string(),
            actor: ActorContext {
                user_id,
                email: None,
                role: "user".to_string(),
                ip_address: None,
                session_id: None,
            },
            resource: None,
            action: ActionDetails {
                operation: "suspicious_activity_detected".to_string(),
                details: serde_json::json!({
                    "activity_type": activity_type,
                    "confidence_score": confidence_score,
                }),
            },
            compliance: ComplianceMetadata {
                gdpr_classification: Some("security_event".to_string()),
                data_subjects_affected: None,
                pii_detected: false,
                legal_hold: false,
            },
            chain: ChainMetadata {
                sequence_number: 0,
                entry_hash: String::new(),
                previous_hash: String::new(),
                merkle_tree_hash: None,
            },
        }
    }
}
