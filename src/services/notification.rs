// tokenization-backend/src/services/notification.rs


// use async_trait::async_trait;
use chrono::{DateTime, Utc};
// use lettre::{
//     message::{header::ContentType, Mailbox, SinglePart},
//     transport::smtp::authentication::Credentials,
//     Message, SmtpTransport, Transport,
// };
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    config::NotificationConfig,
    // utils::errors::{AppError, AppResult},
};

// ------------------------- Traits -------------------------

// Email provider trait
#[async_trait::async_trait]
pub trait EmailProvider {
    async fn send_email(&self, email: &EmailMessage) -> Result<(), NotificationError>;
    async fn send_bulk_email(&self, emails: &[EmailMessage]) -> Result<(), NotificationError>;
}

// Push notification provider trait (async only)
#[async_trait::async_trait]
pub trait PushProvider {
    async fn send_push(&self, notification: &PushNotification) -> Result<(), NotificationError>;
    async fn send_bulk_push(
        &self,
        notifications: &[PushNotification],
    ) -> Result<(), NotificationError>;
}

// ------------------------- Data structures -------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub attachments: Option<Vec<EmailAttachment>>,
    pub template_id: Option<String>,
    pub template_data: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub filename: String,
    pub content_type: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotification {
    pub device_tokens: Vec<String>,
    pub title: String,
    pub body: String,
    pub data: Option<HashMap<String, String>>,
    pub badge_count: Option<u32>,
    pub sound: Option<String>,
    pub category: Option<String>,
    pub platform: NotificationPlatform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPlatform {
    IOs,
    Android,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub id: String,
    pub name: String,
    pub subject_template: String,
    pub html_template: String,
    pub text_template: String,
    pub notification_type: NotificationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Welcome,
    KycApproved,
    KycRejected,
    KycRequiresReview,
    ProjectCreated,
    ProjectApproved,
    ProjectRejected,
    ProjectFunded,
    InvestmentConfirmed,
    InvestmentFailed,
    TokenDeployed,
    WithdrawalProcessed,
    SecurityAlert,
    SystemMaintenance,
    MarketingUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: Uuid,
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub sms_notifications: bool,
    pub marketing_emails: bool,
    pub security_alerts: bool,
    pub transaction_updates: bool,
    pub project_updates: bool,
    pub kyc_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub status: NotificationStatus,
    pub recipient: String,
    pub subject: Option<String>,
    pub content: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub opened_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Push,
    SMS,
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Bounced,
    Opened,
    Clicked,
}

// ------------------------- Error types -------------------------

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("SMTP error: {0}")]
    SmtpError(String),
    #[error("Push notification error: {0}")]
    PushError(String),
    #[error("Template error: {0}")]
    TemplateError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),
}

// ------------------------- Notification Service -------------------------

pub struct NotificationService {
    config: NotificationConfig,
    email_client: Box<dyn EmailProvider + Send + Sync>,
    push_client: Box<dyn PushProvider + Send + Sync>,
}
