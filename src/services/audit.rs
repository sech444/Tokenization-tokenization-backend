use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    models::user::{User, UserRole},
    utils::errors::{AppError, AppResult},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub event_type: AuditEventType,
    pub category: AuditCategory,
    pub severity: AuditSeverity,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub resource_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub action: String,
    pub description: String,
    pub metadata: Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub previous_values: Option<Value>,
    pub new_values: Option<Value>,
    pub status: AuditStatus,
    pub error_message: Option<String>,
    pub compliance_flags: Vec<String>,
    pub retention_period_days: i32,
    pub hash_chain: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    UserLogin,
    UserLogout,
    UserRegistration,
    PasswordChange,
    TwoFactorEnabled,
    TwoFactorDisabled,
    AccountLocked,
    AccountUnlocked,
    PermissionGranted,
    PermissionDenied,
    RoleChanged,
    AccessAttempt,
    ProjectCreated,
    ProjectUpdated,
    ProjectDeleted,
    ProjectApproved,
    ProjectRejected,
    ProjectStatusChanged,
    TokenCreated,
    TokenMinted,
    TokenBurned,
    TokenTransferred,
    TokenizationInitiated,
    TokenizationCompleted,
    TokenizationFailed,
    OrderCreated,
    OrderExecuted,
    OrderCancelled,
    TradeExecuted,
    KycSubmitted,
    KycApproved,
    KycRejected,
    KycDocumentUploaded,
    ComplianceReview,
    PaymentProcessed,
    PaymentFailed,
    WithdrawalRequested,
    WithdrawalProcessed,
    SystemStartup,
    SystemShutdown,
    ConfigurationChanged,
    BackupCreated,
    DatabaseMigration,
    SecurityBreach,
    SuspiciousActivity,
    RateLimitExceeded,
    InvalidTokenAccess,
    DataExport,
    DataDeletion,
    AdminActionPerformed,
    UserDataAccessed,
    SystemConfigChanged,
    ReportsGenerated,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditCategory {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    Financial,
    Compliance,
    Security,
    System,
    Performance,
    Business,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditStatus {
    Success,
    Failed,
    Pending,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub event_types: Option<Vec<AuditEventType>>,
    pub categories: Option<Vec<AuditCategory>>,
    pub user_ids: Option<Vec<Uuid>>,
    pub resource_ids: Option<Vec<Uuid>>,
    pub severity_levels: Option<Vec<AuditSeverity>>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub ip_addresses: Option<Vec<String>>,
    pub compliance_flags: Option<Vec<String>>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub events: Vec<AuditEvent>,
    pub total_count: i64,
    pub summary: AuditSummary,
    pub compliance_status: ComplianceStatus,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total_events: i64,
    pub events_by_category: HashMap<AuditCategory, i64>,
    pub events_by_severity: HashMap<AuditSeverity, i64>,
    pub unique_users: i64,
    pub unique_ips: i64,
    pub success_rate: f64,
    pub top_actions: Vec<(String, i64)>,
    pub compliance_violations: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub sox_compliant: bool,
    pub gdpr_compliant: bool,
    pub pci_compliant: bool,
    pub iso27001_compliant: bool,
    pub retention_policy_enforced: bool,
    pub violations: Vec<String>,
    pub recommendations: Vec<String>,
}

pub struct AuditService {
    db: PgPool,
    hash_secret: String,
    default_retention_days: i32,
}

impl AuditService {
    pub fn new(db: PgPool, hash_secret: String) -> Self {
        Self {
            db,
            hash_secret,
            default_retention_days: 2555, // ~7 years
        }
    }

    /// Insert an audit event
    pub async fn log_event(&self, mut event: AuditEvent) -> AppResult<Uuid> {
        if event.id == Uuid::nil() {
            event.id = Uuid::new_v4();
        }
        event.created_at = Utc::now();
        event.retention_period_days = self.default_retention_days;
        event.hash_chain = Some(self.generate_hash_chain(&event).await?);

        let query = r#"
            INSERT INTO audit_logs (
                id, event_type, category, severity, user_id, session_id,
                resource_id, resource_type, action, description, metadata,
                ip_address, user_agent, request_id, previous_values, new_values,
                status, error_message, compliance_flags, retention_period_days,
                hash_chain, created_at
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,
                $12,$13,$14,$15,$16,$17,$18,$19,$20,
                $21,$22
            )
        "#;

        sqlx::query(query)
            .bind(event.id)
            .bind(serde_json::to_string(&event.event_type)?)
            .bind(serde_json::to_string(&event.category)?)
            .bind(serde_json::to_string(&event.severity)?)
            .bind(event.user_id)
            .bind(event.session_id)
            .bind(event.resource_id)
            .bind(event.resource_type)
            .bind(event.action)
            .bind(event.description)
            .bind(event.metadata)
            .bind(event.ip_address)
            .bind(event.user_agent)
            .bind(event.request_id)
            .bind(event.previous_values)
            .bind(event.new_values)
            .bind(serde_json::to_string(&event.status)?)
            .bind(event.error_message)
            .bind(event.compliance_flags)
            .bind(event.retention_period_days)
            .bind(event.hash_chain)
            .bind(event.created_at)
            .execute(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to log audit event: {}", e);
                AppError::InternalServerError("Failed to log audit event".to_string())
            })?;

        info!("Audit event {} logged", event.id);
        Ok(event.id)
    }

    async fn generate_hash_chain(&self, event: &AuditEvent) -> AppResult<String> {
        let previous_hash: Option<String> = sqlx::query_scalar(
            "SELECT hash_chain FROM audit_logs ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch previous hash: {}", e);
            AppError::InternalServerError("Hash chain generation failed".to_string())
        })?;

        let previous_hash = previous_hash.unwrap_or_else(|| "genesis".to_string());

        let hash_data = format!(
            "{}{}{}{}{}{}{}",
            previous_hash,
            event.id,
            serde_json::to_string(&event.event_type).unwrap_or_default(),
            event.action,
            event.created_at.timestamp(),
            serde_json::to_string(&event.metadata).unwrap_or_default(),
            self.hash_secret
        );

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(hash_data.as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }

    pub async fn verify_integrity(&self, _start: Option<Uuid>, _end: Option<Uuid>) -> AppResult<bool> {
        let events = sqlx::query("SELECT id, hash_chain, created_at FROM audit_logs ORDER BY created_at")
            .map(|row: sqlx::postgres::PgRow| {
                (
                    row.get::<Uuid, _>("id"),
                    row.get::<Option<String>, _>("hash_chain"),
                    row.get::<DateTime<Utc>, _>("created_at"),
                )
            })
            .fetch_all(&self.db)
            .await?;

        if events.len() < 2 {
            return Ok(true);
        }

        for window in events.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            let expected = self
                .compute_expected_hash(
                    prev.1.as_deref().unwrap_or("genesis"),
                    curr.0,
                    curr.2,
                )
                .await?;

            if curr.1.as_deref() != Some(&expected) {
                warn!("Hash chain integrity violation at {}", curr.0);
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn compute_expected_hash(
        &self,
        prev_hash: &str,
        event_id: Uuid,
        created_at: DateTime<Utc>,
    ) -> AppResult<String> {
        let data = format!("{}{}{}{}", prev_hash, event_id, created_at.timestamp(), self.hash_secret);
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }

    pub async fn cleanup_old_logs(&self) -> AppResult<i64> {
        let deleted: Option<i64> = sqlx::query_scalar(
            r#"
            DELETE FROM audit_logs
            WHERE created_at + INTERVAL '1 day' * retention_period_days < NOW()
            RETURNING COUNT(*)
            "#
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to cleanup audit logs: {}", e);
            AppError::InternalServerError("Cleanup failed".to_string())
        })?;

        Ok(deleted.unwrap_or(0))
    }
}

// === Helper loggers ===
impl AuditService {
    pub async fn log_user_auth(
        &self,
        user_id: Uuid,
        event_type: AuditEventType,
        ip: Option<String>,
        success: bool,
    ) -> AppResult<()> {
        let ev = AuditEvent {
            id: Uuid::new_v4(),
            event_type,
            category: AuditCategory::Authentication,
            severity: if success { AuditSeverity::Low } else { AuditSeverity::Medium },
            user_id: Some(user_id),
            session_id: None,
            resource_id: None,
            resource_type: None,
            action: "authentication".to_string(),
            description: format!("User {} authentication", if success { "successful" } else { "failed" }),
            metadata: json!({"success": success}),
            ip_address: ip,
            user_agent: None,
            request_id: None,
            previous_values: None,
            new_values: None,
            status: if success { AuditStatus::Success } else { AuditStatus::Failed },
            error_message: None,
            compliance_flags: vec![],
            retention_period_days: self.default_retention_days,
            hash_chain: None,
            created_at: Utc::now(),
        };
        self.log_event(ev).await?;
        Ok(())
    }
}
