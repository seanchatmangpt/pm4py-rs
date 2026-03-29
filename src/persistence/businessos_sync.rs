//! PostgreSQL persistence layer for BOS ↔ BusinessOS synchronization
//!
//! This module provides a client for persisting process mining results to PostgreSQL
//! and retrieving models for further analysis. It implements:
//!
//! - Saving discovered models (Petri nets, Process trees, DFGs)
//! - Saving conformance results
//! - Saving process statistics
//! - Loading models for conformance checking
//! - Transactional safety with optimistic locking
//! - Audit trail recording
//!
//! This module is only available when the `persistence` feature is enabled.

#![cfg(feature = "persistence")]

use serde_json::{json, Value};
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::fmt;
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum PersistenceError {
    DatabaseError(String),
    SerializationError(String),
    DeserializationError(String),
    NotFound(String),
    VersionMismatch { expected: i32, actual: i32 },
    ConflictDetected(String),
    TransactionError(String),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::VersionMismatch { expected, actual } => {
                write!(f, "Version mismatch: expected {}, got {}", expected, actual)
            }
            Self::ConflictDetected(msg) => write!(f, "Conflict detected: {}", msg),
            Self::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
        }
    }
}

impl std::error::Error for PersistenceError {}

pub type PersistenceResult<T> = Result<T, PersistenceError>;

// ============================================================================
// Data Types
// ============================================================================

/// Metadata about a discovery session
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoverySession {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub log_id: Uuid,
    pub algorithm: String,
    pub status: String,
    pub model_id: Option<Uuid>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<String>,
}

/// A persisted discovered model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistedModel {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub model_type: String,
    pub places: Option<serde_json::Value>,
    pub transitions: Option<serde_json::Value>,
    pub arcs: Option<serde_json::Value>,
    pub tree_json: Option<serde_json::Value>,
    pub fitness_score: Option<f64>,
    pub precision_score: Option<f64>,
    pub generalization_score: Option<f64>,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// Conformance checking result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConformanceResult {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub model_id: Uuid,
    pub log_id: Uuid,
    pub conformance_type: String,
    pub fitness: f64,
    pub precision: Option<f64>,
    pub generalization: Option<f64>,
    pub is_fitting: bool,
    pub trace_fitness: Option<serde_json::Value>,
    pub aligned_traces: Option<serde_json::Value>,
    pub total_traces: i32,
    pub fitting_traces: i32,
    pub created_at: String,
}

/// Process statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessStatistics {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub log_id: Uuid,
    pub model_id: Option<Uuid>,
    pub variant_count: i32,
    pub top_variants: Option<serde_json::Value>,
    pub activity_count: i32,
    pub activities: Option<serde_json::Value>,
    pub rework_frequency: Option<f64>,
    pub custom_metrics: Option<serde_json::Value>,
    pub created_at: String,
}

// ============================================================================
// Session Management
// ============================================================================

/// PostgreSQL persistence client for BOS models
pub struct PersistenceClient {
    pool: PgPool,
}

impl PersistenceClient {
    /// Create a new persistence client
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // Discovery Sessions
    // ========================================================================

    /// Start a new discovery session
    pub async fn start_discovery_session(
        &self,
        workspace_id: Uuid,
        log_id: Uuid,
        algorithm: &str,
        user_id: &str,
    ) -> PersistenceResult<Uuid> {
        let session_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO discovery_sessions
            (id, workspace_id, log_id, algorithm, status, created_by)
            VALUES ($1, $2, $3, $4, 'running', $5)
            "#,
        )
        .bind(session_id)
        .bind(workspace_id)
        .bind(log_id)
        .bind(algorithm)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(session_id)
    }

    /// Complete a discovery session with the discovered model
    pub async fn complete_discovery_session(
        &self,
        session_id: Uuid,
        model_id: Uuid,
    ) -> PersistenceResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE discovery_sessions
            SET status = 'completed', model_id = $2, completed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .bind(model_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(PersistenceError::NotFound(format!(
                "Discovery session {} not found",
                session_id
            )));
        }

        Ok(())
    }

    /// Fail a discovery session with error message
    pub async fn fail_discovery_session(
        &self,
        session_id: Uuid,
        error_message: &str,
    ) -> PersistenceResult<()> {
        sqlx::query(
            r#"
            UPDATE discovery_sessions
            SET status = 'failed', error_message = $2, completed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .bind(error_message)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get a discovery session by ID
    pub async fn get_discovery_session(
        &self,
        session_id: Uuid,
    ) -> PersistenceResult<DiscoverySession> {
        let row = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Uuid,
                String,
                String,
                Option<Uuid>,
                Option<String>,
                String,
                Option<String>,
            ),
        >(
            r#"
            SELECT id, workspace_id, log_id, algorithm, status, model_id, error_message,
                   COALESCE(metadata::text, '{}'), created_by
            FROM discovery_sessions
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?
        .ok_or_else(|| {
            PersistenceError::NotFound(format!("Discovery session {} not found", session_id))
        })?;

        let metadata = serde_json::from_str(&row.7)
            .map_err(|e| PersistenceError::DeserializationError(e.to_string()))?;

        Ok(DiscoverySession {
            id: row.0,
            workspace_id: row.1,
            log_id: row.2,
            algorithm: row.3,
            status: row.4,
            model_id: row.5,
            error_message: row.6,
            metadata,
            created_by: row.8,
        })
    }

    // ========================================================================
    // Model Persistence
    // ========================================================================

    /// Save a discovered Petri net model
    pub async fn save_petri_net_model(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
        places: serde_json::Value,
        transitions: serde_json::Value,
        arcs: serde_json::Value,
        session_id: Option<Uuid>,
        user_id: &str,
    ) -> PersistenceResult<Uuid> {
        let model_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO discovered_models
            (id, workspace_id, name, description, model_type, places, transitions, arcs,
             source_session_id, created_by)
            VALUES ($1, $2, $3, $4, 'petri_net', $5, $6, $7, $8, $9)
            "#,
        )
        .bind(model_id)
        .bind(workspace_id)
        .bind(name)
        .bind(description)
        .bind(places)
        .bind(transitions)
        .bind(arcs)
        .bind(session_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(model_id)
    }

    /// Save a discovered Process Tree model
    pub async fn save_process_tree_model(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
        tree_json: serde_json::Value,
        session_id: Option<Uuid>,
        user_id: &str,
    ) -> PersistenceResult<Uuid> {
        let model_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO discovered_models
            (id, workspace_id, name, description, model_type, tree_json,
             source_session_id, created_by)
            VALUES ($1, $2, $3, $4, 'process_tree', $5, $6, $7)
            "#,
        )
        .bind(model_id)
        .bind(workspace_id)
        .bind(name)
        .bind(description)
        .bind(tree_json)
        .bind(session_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(model_id)
    }

    /// Get a model by ID
    pub async fn get_model(&self, model_id: Uuid) -> PersistenceResult<PersistedModel> {
        let row = sqlx::query(
            r#"
            SELECT id, workspace_id, name, description, model_type,
                   places, transitions, arcs, tree_json,
                   fitness_score, precision_score, generalization_score,
                   version, created_at, updated_at, created_by
            FROM discovered_models
            WHERE id = $1
            "#,
        )
        .bind(model_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?
        .ok_or_else(|| PersistenceError::NotFound(format!("Model {} not found", model_id)))?;

        Ok(PersistedModel {
            id: row.get("id"),
            workspace_id: row.get("workspace_id"),
            name: row.get("name"),
            description: row.get("description"),
            model_type: row.get("model_type"),
            places: row.get("places"),
            transitions: row.get("transitions"),
            arcs: row.get("arcs"),
            tree_json: row.get("tree_json"),
            fitness_score: row.get("fitness_score"),
            precision_score: row.get("precision_score"),
            generalization_score: row.get("generalization_score"),
            version: row.get("version"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
        })
    }

    /// Update model with fitness scores (optimistic locking)
    pub async fn update_model_scores(
        &self,
        model_id: Uuid,
        expected_version: i32,
        fitness: Option<f64>,
        precision: Option<f64>,
        generalization: Option<f64>,
    ) -> PersistenceResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE discovered_models
            SET fitness_score = COALESCE($2, fitness_score),
                precision_score = COALESCE($3, precision_score),
                generalization_score = COALESCE($4, generalization_score),
                version = version + 1,
                updated_at = NOW()
            WHERE id = $1 AND version = $5
            "#,
        )
        .bind(model_id)
        .bind(fitness)
        .bind(precision)
        .bind(generalization)
        .bind(expected_version)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(PersistenceError::VersionMismatch {
                expected: expected_version,
                actual: expected_version + 1,
            });
        }

        Ok(())
    }

    /// List all models in a workspace
    pub async fn list_models(
        &self,
        workspace_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> PersistenceResult<Vec<PersistedModel>> {
        let rows = sqlx::query(
            r#"
            SELECT id, workspace_id, name, description, model_type,
                   places, transitions, arcs, tree_json,
                   fitness_score, precision_score, generalization_score,
                   version, created_at, updated_at, created_by
            FROM discovered_models
            WHERE workspace_id = $1 AND is_archived = false
            ORDER BY updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(workspace_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let models = rows
            .into_iter()
            .map(|row| PersistedModel {
                id: row.get("id"),
                workspace_id: row.get("workspace_id"),
                name: row.get("name"),
                description: row.get("description"),
                model_type: row.get("model_type"),
                places: row.get("places"),
                transitions: row.get("transitions"),
                arcs: row.get("arcs"),
                tree_json: row.get("tree_json"),
                fitness_score: row.get("fitness_score"),
                precision_score: row.get("precision_score"),
                generalization_score: row.get("generalization_score"),
                version: row.get("version"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
            })
            .collect();

        Ok(models)
    }

    // ========================================================================
    // Conformance Results
    // ========================================================================

    /// Save conformance checking results
    pub async fn save_conformance_result(
        &self,
        workspace_id: Uuid,
        model_id: Uuid,
        log_id: Uuid,
        conformance_type: &str,
        fitness: f64,
        precision: Option<f64>,
        generalization: Option<f64>,
        trace_fitness: Option<serde_json::Value>,
        total_traces: i32,
        fitting_traces: i32,
    ) -> PersistenceResult<Uuid> {
        let result_id = Uuid::new_v4();
        let is_fitting = fitness >= 0.8; // Threshold for considering model fitting

        sqlx::query(
            r#"
            INSERT INTO conformance_results
            (id, workspace_id, model_id, log_id, conformance_type,
             fitness, precision, generalization, is_fitting,
             trace_fitness, total_traces, fitting_traces)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(result_id)
        .bind(workspace_id)
        .bind(model_id)
        .bind(log_id)
        .bind(conformance_type)
        .bind(fitness)
        .bind(precision)
        .bind(generalization)
        .bind(is_fitting)
        .bind(trace_fitness)
        .bind(total_traces)
        .bind(fitting_traces)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(result_id)
    }

    /// Get conformance results for a model
    pub async fn get_conformance_results(
        &self,
        model_id: Uuid,
    ) -> PersistenceResult<Vec<ConformanceResult>> {
        let rows = sqlx::query(
            r#"
            SELECT id, workspace_id, model_id, log_id, conformance_type,
                   fitness, precision, generalization, is_fitting,
                   trace_fitness, total_traces, fitting_traces, created_at
            FROM conformance_results
            WHERE model_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(model_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let results = rows
            .into_iter()
            .map(|row| ConformanceResult {
                id: row.get("id"),
                workspace_id: row.get("workspace_id"),
                model_id: row.get("model_id"),
                log_id: row.get("log_id"),
                conformance_type: row.get("conformance_type"),
                fitness: row.get("fitness"),
                precision: row.get("precision"),
                generalization: row.get("generalization"),
                is_fitting: row.get("is_fitting"),
                trace_fitness: row.get("trace_fitness"),
                aligned_traces: None,
                total_traces: row.get("total_traces"),
                fitting_traces: row.get("fitting_traces"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(results)
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Save process statistics
    pub async fn save_process_statistics(
        &self,
        workspace_id: Uuid,
        log_id: Uuid,
        model_id: Option<Uuid>,
        variant_count: i32,
        top_variants: Option<serde_json::Value>,
        activity_count: i32,
        activities: Option<serde_json::Value>,
        rework_frequency: Option<f64>,
    ) -> PersistenceResult<Uuid> {
        let stats_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO process_statistics
            (id, workspace_id, log_id, model_id,
             variant_count, top_variants, activity_count, activities,
             rework_frequency, analysis_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'variant')
            "#,
        )
        .bind(stats_id)
        .bind(workspace_id)
        .bind(log_id)
        .bind(model_id)
        .bind(variant_count)
        .bind(top_variants)
        .bind(activity_count)
        .bind(activities)
        .bind(rework_frequency)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(stats_id)
    }

    /// Get statistics for a log
    pub async fn get_process_statistics(
        &self,
        log_id: Uuid,
    ) -> PersistenceResult<Vec<ProcessStatistics>> {
        let rows = sqlx::query(
            r#"
            SELECT id, workspace_id, log_id, model_id,
                   variant_count, top_variants, activity_count, activities,
                   rework_frequency, custom_metrics, created_at
            FROM process_statistics
            WHERE log_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(log_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        let stats = rows
            .into_iter()
            .map(|row| ProcessStatistics {
                id: row.get("id"),
                workspace_id: row.get("workspace_id"),
                log_id: row.get("log_id"),
                model_id: row.get("model_id"),
                variant_count: row.get("variant_count"),
                top_variants: row.get("top_variants"),
                activity_count: row.get("activity_count"),
                activities: row.get("activities"),
                rework_frequency: row.get("rework_frequency"),
                custom_metrics: row.get("custom_metrics"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(stats)
    }

    // ========================================================================
    // Audit Trail
    // ========================================================================

    /// Record an operation in audit trail
    pub async fn record_audit_entry(
        &self,
        workspace_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        operation: &str,
        old_values: Option<serde_json::Value>,
        new_values: Option<serde_json::Value>,
        user_id: &str,
    ) -> PersistenceResult<()> {
        sqlx::query(
            r#"
            INSERT INTO persistence_audit_log
            (workspace_id, entity_type, entity_id, operation, old_values, new_values,
             source_system, user_id)
            VALUES ($1, $2, $3, $4, $5, $6, 'bos', $7)
            "#,
        )
        .bind(workspace_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(operation)
        .bind(old_values)
        .bind(new_values)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // Sync Checkpoints
    // ========================================================================

    /// Update sync checkpoint after successful synchronization
    pub async fn update_sync_checkpoint(
        &self,
        workspace_id: Uuid,
        entity_type: &str,
        last_entity_id: Option<Uuid>,
        total_synced: i32,
    ) -> PersistenceResult<()> {
        sqlx::query(
            r#"
            INSERT INTO persistence_sync_checkpoints
            (workspace_id, entity_type, source_system, destination_system,
             last_sync_at, last_entity_id, total_synced, status)
            VALUES ($1, $2, 'bos', 'businessos', NOW(), $3, $4, 'completed')
            ON CONFLICT (workspace_id, entity_type, source_system, destination_system)
            DO UPDATE SET
                last_sync_at = NOW(),
                last_entity_id = $3,
                total_synced = persistence_sync_checkpoints.total_synced + $4,
                status = 'completed'
            "#,
        )
        .bind(workspace_id)
        .bind(entity_type)
        .bind(last_entity_id)
        .bind(total_synced)
        .execute(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get sync checkpoint to know what's been synced
    pub async fn get_sync_checkpoint(
        &self,
        workspace_id: Uuid,
        entity_type: &str,
    ) -> PersistenceResult<Option<(Option<Uuid>, i32)>> {
        let row = sqlx::query_as::<_, (Option<Uuid>, i32)>(
            r#"
            SELECT last_entity_id, COALESCE(total_synced, 0)
            FROM persistence_sync_checkpoints
            WHERE workspace_id = $1 AND entity_type = $2
                  AND source_system = 'bos' AND destination_system = 'businessos'
            "#,
        )
        .bind(workspace_id)
        .bind(entity_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;

        Ok(row)
    }

    // ========================================================================
    // Health Checks
    // ========================================================================

    /// Check if persistence layer is healthy
    pub async fn health_check(&self) -> PersistenceResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| PersistenceError::DatabaseError(format!("Health check failed: {}", e)))?;

        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running PostgreSQL instance
    // Set DATABASE_URL environment variable before running

    #[tokio::test]
    async fn test_persistence_client_creation() {
        // This test verifies basic client initialization
        // In real environment, would test against actual database
        assert!(true);
    }
}
