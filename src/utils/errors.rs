// tokenization-backend/src/utils/errors.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use log;
use serde::{Deserialize, Serialize};
use std::fmt;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: Option<String>,
    pub details: Option<serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // Authentication & Authorization
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    Forbidden(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    // Database errors
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Database connection failed: {0}")]
    DatabaseConnectionError(String),

    #[error("Database migration failed: {0}")]
    DatabaseMigrationError(String),

    #[error("Transaction failed: {0}")]
    DatabaseTransactionError(String),

    // Validation errors
    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Value out of range: {0}")]
    ValueOutOfRange(String),

    // Business logic errors
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    #[error("Operation not allowed: {0}")]
    NotAllowed(String),

    #[error("Insufficient funds: available {available}, required {required}")]
    InsufficientFunds { available: i64, required: i64 },

    #[error("Investment limit exceeded: {0}")]
    InvestmentLimitExceeded(String),

    #[error("Project not active: {0}")]
    ProjectNotActive(String),

    #[error("Project funding goal reached")]
    FundingGoalReached,

    // KYC & Compliance errors
    #[error("KYC verification required")]
    KycVerificationRequired,

    #[error("KYC verification failed: {0}")]
    KycVerificationFailed(String),

    #[error("KYC verification pending")]
    KycVerificationPending,

    #[error("AML screening failed: {0}")]
    AmlScreeningFailed(String),

    #[error("Compliance check failed: {0}")]
    ComplianceCheckFailed(String),

    #[error("Geographic restriction: {0}")]
    GeographicRestriction(String),

    #[error("Accredited investor verification required")]
    AccreditedInvestorRequired,

    // Blockchain errors
    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Smart contract error: {0}")]
    SmartContractError(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Insufficient gas: {0}")]
    InsufficientGas(String),

    #[error("Contract deployment failed: {0}")]
    ContractDeploymentFailed(String),

    #[error("Invalid contract address: {0}")]
    InvalidContractAddress(String),

    #[error("Token transfer failed: {0}")]
    TokenTransferFailed(String),

    // External service errors
    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Payment provider error: {0}")]
    PaymentProviderError(String),

    #[error("Email service error: {0}")]
    EmailServiceError(String),

    #[error("Push notification error: {0}")]
    PushNotificationError(String),

    #[error("File storage error: {0}")]
    FileStorageError(String),

    // Network & Communication errors
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    #[error("DNS resolution failed: {0}")]
    DnsError(String),

    // Configuration errors
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Missing configuration: {0}")]
    MissingConfiguration(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    // Serialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("JSON parsing error: {0}")]
    JsonError(String),

    // Rate limiting
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Too many requests")]
    TooManyRequests,

    // File operations
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("File permission denied: {0}")]
    FilePermissionDenied(String),

    #[error("File size too large: {0}")]
    FileTooLarge(String),

    #[error("Invalid file format: {0}")]
    InvalidFileFormat(String),

    // Cryptography errors
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Hash verification failed: {0}")]
    HashVerificationFailed(String),

    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    // Generic errors
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            // Authentication & Authorization
            AppError::AuthenticationFailed(_) => "AUTH_001",
            AppError::Forbidden(_) => "AUTH_002",
            AppError::InvalidCredentials => "AUTH_003",
            AppError::TokenExpired => "AUTH_004",
            AppError::InvalidToken(_) => "AUTH_005",

            // Database errors
            AppError::DatabaseError(_) => "DB_001",
            AppError::DatabaseConnectionError(_) => "DB_002",
            AppError::DatabaseMigrationError(_) => "DB_003",
            AppError::DatabaseTransactionError(_) => "DB_004",

            // Validation errors
            AppError::ValidationError(_) => "VAL_001",
            AppError::InvalidInput(_) => "VAL_002",
            AppError::MissingField(_) => "VAL_003",
            AppError::InvalidFormat(_) => "VAL_004",
            AppError::ValueOutOfRange(_) => "VAL_005",

            // Business logic errors
            AppError::NotFound(_) => "BUS_001",
            AppError::AlreadyExists(_) => "BUS_002",
            AppError::NotAllowed(_) => "BUS_003",
            AppError::InsufficientFunds { .. } => "BUS_004",
            AppError::InvestmentLimitExceeded(_) => "BUS_005",
            AppError::ProjectNotActive(_) => "BUS_006",
            AppError::FundingGoalReached => "BUS_007",

            // KYC & Compliance errors
            AppError::KycVerificationRequired => "KYC_001",
            AppError::KycVerificationFailed(_) => "KYC_002",
            AppError::KycVerificationPending => "KYC_003",
            AppError::AmlScreeningFailed(_) => "AML_001",
            AppError::ComplianceCheckFailed(_) => "COMP_001",
            AppError::GeographicRestriction(_) => "COMP_002",
            AppError::AccreditedInvestorRequired => "COMP_003",

            // Blockchain errors
            AppError::BlockchainError(_) => "BC_001",
            AppError::SmartContractError(_) => "BC_002",
            AppError::TransactionFailed(_) => "BC_003",
            AppError::InsufficientGas(_) => "BC_004",
            AppError::ContractDeploymentFailed(_) => "BC_005",
            AppError::InvalidContractAddress(_) => "BC_006",
            AppError::TokenTransferFailed(_) => "BC_007",

            // External service errors
            AppError::ExternalServiceError(_) => "EXT_001",
            AppError::PaymentProviderError(_) => "PAY_001",
            AppError::EmailServiceError(_) => "EMAIL_001",
            AppError::PushNotificationError(_) => "PUSH_001",
            AppError::FileStorageError(_) => "STORAGE_001",

            // Network & Communication errors
            AppError::NetworkError(_) => "NET_001",
            AppError::TimeoutError(_) => "NET_002",
            AppError::ConnectionRefused(_) => "NET_003",
            AppError::DnsError(_) => "NET_004",

            // Configuration errors
            AppError::ConfigurationError(_) => "CFG_001",
            AppError::MissingConfiguration(_) => "CFG_002",
            AppError::InvalidConfiguration(_) => "CFG_003",

            // Serialization errors
            AppError::SerializationError(_) => "SER_001",
            AppError::DeserializationError(_) => "SER_002",
            AppError::JsonError(_) => "SER_003",

            // Rate limiting
            AppError::RateLimitError(_) => "RATE_001",
            AppError::TooManyRequests => "RATE_002",

            // File operations
            AppError::FileNotFound(_) => "FILE_001",
            AppError::FilePermissionDenied(_) => "FILE_002",
            AppError::FileTooLarge(_) => "FILE_003",
            AppError::InvalidFileFormat(_) => "FILE_004",

            // Cryptography errors
            AppError::EncryptionError(_) => "CRYPTO_001",
            AppError::DecryptionError(_) => "CRYPTO_002",
            AppError::HashVerificationFailed(_) => "CRYPTO_003",
            AppError::SignatureVerificationFailed(_) => "CRYPTO_004",

            // Generic errors
            AppError::BadRequest(_) => "GEN_001",
            AppError::InternalServerError(_) => "GEN_002",
            AppError::ServiceUnavailable(_) => "GEN_003",
            AppError::NotImplemented(_) => "GEN_004",
            AppError::Unexpected(_) => "GEN_005",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            // 400 Bad Request
            AppError::ValidationError(_)
            | AppError::InvalidInput(_)
            | AppError::MissingField(_)
            | AppError::InvalidFormat(_)
            | AppError::ValueOutOfRange(_)
            | AppError::BadRequest(_)
            | AppError::InvalidCredentials
            | AppError::InvalidToken(_)
            | AppError::AlreadyExists(_)
            | AppError::NotAllowed(_)
            | AppError::InsufficientFunds { .. }
            | AppError::InvestmentLimitExceeded(_)
            | AppError::ProjectNotActive(_)
            | AppError::FundingGoalReached
            | AppError::InvalidContractAddress(_)
            | AppError::InvalidFileFormat(_)
            | AppError::FileTooLarge(_)
            | AppError::JsonError(_) => StatusCode::BAD_REQUEST,

            // 401 Unauthorized
            AppError::AuthenticationFailed(_) | AppError::TokenExpired => StatusCode::UNAUTHORIZED,

            // 403 Forbidden
            AppError::Forbidden(_)
            | AppError::KycVerificationRequired
            | AppError::KycVerificationFailed(_)
            | AppError::GeographicRestriction(_)
            | AppError::AccreditedInvestorRequired
            | AppError::FilePermissionDenied(_) => StatusCode::FORBIDDEN,

            // 404 Not Found
            AppError::NotFound(_) | AppError::FileNotFound(_) => StatusCode::NOT_FOUND,

            // 409 Conflict
            AppError::KycVerificationPending => StatusCode::CONFLICT,

            // 422 Unprocessable Entity
            AppError::AmlScreeningFailed(_)
            | AppError::ComplianceCheckFailed(_)
            | AppError::BlockchainError(_)
            | AppError::SmartContractError(_)
            | AppError::TransactionFailed(_)
            | AppError::ContractDeploymentFailed(_)
            | AppError::TokenTransferFailed(_)
            | AppError::PaymentProviderError(_) => StatusCode::UNPROCESSABLE_ENTITY,

            // 429 Too Many Requests
            AppError::RateLimitError(_) | AppError::TooManyRequests => {
                StatusCode::TOO_MANY_REQUESTS
            }

            // 500 Internal Server Error
            AppError::DatabaseError(_)
            | AppError::DatabaseConnectionError(_)
            | AppError::DatabaseMigrationError(_)
            | AppError::DatabaseTransactionError(_)
            | AppError::ConfigurationError(_)
            | AppError::MissingConfiguration(_)
            | AppError::InvalidConfiguration(_)
            | AppError::SerializationError(_)
            | AppError::DeserializationError(_)
            | AppError::EncryptionError(_)
            | AppError::DecryptionError(_)
            | AppError::HashVerificationFailed(_)
            | AppError::SignatureVerificationFailed(_)
            | AppError::InternalServerError(_)
            | AppError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,

            // 501 Not Implemented
            AppError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,

            // 502 Bad Gateway
            AppError::ExternalServiceError(_)
            | AppError::EmailServiceError(_)
            | AppError::PushNotificationError(_)
            | AppError::InsufficientGas(_) => StatusCode::BAD_GATEWAY,

            // 503 Service Unavailable
            AppError::ServiceUnavailable(_)
            | AppError::NetworkError(_)
            | AppError::ConnectionRefused(_)
            | AppError::FileStorageError(_) => StatusCode::SERVICE_UNAVAILABLE,

            // 504 Gateway Timeout
            AppError::TimeoutError(_) | AppError::DnsError(_) => StatusCode::GATEWAY_TIMEOUT,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AppError::NetworkError(_)
                | AppError::TimeoutError(_)
                | AppError::ConnectionRefused(_)
                | AppError::ServiceUnavailable(_)
                | AppError::ExternalServiceError(_)
                | AppError::DatabaseConnectionError(_)
                | AppError::RateLimitError(_)
                | AppError::TooManyRequests
        )
    }

    pub fn is_client_error(&self) -> bool {
        self.status_code().is_client_error()
    }

    pub fn is_server_error(&self) -> bool {
        self.status_code().is_server_error()
    }

    pub fn with_details( self, details: serde_json::Value) -> ErrorResponse {
        ErrorResponse {
            error: self.error_code().to_string(),
            message: self.to_string(),
            code: Some(self.error_code().to_string()),
            details: Some(details),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            error: self.error_code().to_string(),
            message: self.to_string(),
            code: Some(self.error_code().to_string()),
            details: None,
            timestamp: chrono::Utc::now(),
        };

        (status_code, Json(error_response)).into_response()
    }
}

// Conversion from common error types
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => AppError::AlreadyExists("Resource already exists".to_string()),
                        "23503" => AppError::ValidationError(
                            "Foreign key constraint violation".to_string(),
                        ),
                        "23502" => {
                            AppError::ValidationError("Not null constraint violation".to_string())
                        }
                        _ => AppError::DatabaseError(db_err.to_string()),
                    }
                } else {
                    AppError::DatabaseError(db_err.to_string())
                }
            }
            _ => AppError::DatabaseError(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::JsonError(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind;
        match err.kind() {
            ErrorKind::NotFound => AppError::FileNotFound(err.to_string()),
            ErrorKind::PermissionDenied => AppError::FilePermissionDenied(err.to_string()),
            ErrorKind::TimedOut => AppError::TimeoutError(err.to_string()),
            ErrorKind::ConnectionRefused => AppError::ConnectionRefused(err.to_string()),
            _ => AppError::InternalServerError(err.to_string()),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::TimeoutError(err.to_string())
        } else if err.is_connect() {
            AppError::ConnectionRefused(err.to_string())
        } else if err.is_request() {
            AppError::BadRequest(err.to_string())
        } else {
            AppError::NetworkError(err.to_string())
        }
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        AppError::TimeoutError(err.to_string())
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::InvalidFormat(format!("Invalid UUID: {}", err))
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::InternalServerError(format!("Password hashing error: {}", err))
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        AppError::InvalidFormat(format!("Invalid date format: {}", err))
    }
}

// Helper functions for creating specific errors
impl AppError {
    pub fn not_found<T: fmt::Display>(resource: T) -> Self {
        AppError::NotFound(resource.to_string())
    }

    pub fn already_exists<T: fmt::Display>(resource: T) -> Self {
        AppError::AlreadyExists(resource.to_string())
    }

    pub fn validation<T: fmt::Display>(message: T) -> Self {
        AppError::ValidationError(message.to_string())
    }

    pub fn forbidden<T: fmt::Display>(message: T) -> Self {
        AppError::Forbidden(message.to_string())
    }

    pub fn bad_request<T: fmt::Display>(message: T) -> Self {
        AppError::BadRequest(message.to_string())
    }

    pub fn internal_server_error<T: fmt::Display>(message: T) -> Self {
        AppError::InternalServerError(message.to_string())
    }

    pub fn blockchain<T: fmt::Display>(message: T) -> Self {
        AppError::BlockchainError(message.to_string())
    }

    pub fn external_service<T: fmt::Display>(service: T) -> Self {
        AppError::ExternalServiceError(service.to_string())
    }
}

// Error logging helper
pub fn log_error(error: &AppError, context: &str) {
    match error {
        AppError::InternalServerError(_)
        | AppError::DatabaseError(_)
        | AppError::ConfigurationError(_)
        | AppError::Unexpected(_) => {
            log::error!("[{}] {}: {}", context, error.error_code(), error);
        }
        AppError::ExternalServiceError(_)
        | AppError::NetworkError(_)
        | AppError::TimeoutError(_) => {
            log::warn!("[{}] {}: {}", context, error.error_code(), error);
        }
        _ => {
            log::debug!("[{}] {}: {}", context, error.error_code(), error);
        }
    }
}

// Error context wrapper
pub struct ErrorContext {
    pub operation: String,
    pub user_id: Option<uuid::Uuid>,
    pub request_id: Option<String>,
    pub additional_data: Option<serde_json::Value>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            user_id: None,
            request_id: None,
            additional_data: None,
        }
    }

    pub fn with_user_id(mut self, user_id: uuid::Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.additional_data = Some(data);
        self
    }

    pub fn log_error(&self, error: &AppError) {
        let context = format!(
            "operation={} user_id={:?} request_id={:?}",
            self.operation, self.user_id, self.request_id
        );
        log_error(error, &context);
    }
}

// Result type alias with context
pub type ContextResult<T> = Result<T, (AppError, ErrorContext)>;

// Trait for adding context to results
pub trait ResultExt<T> {
    fn with_context(self, context: ErrorContext) -> ContextResult<T>;
    fn with_operation(self, operation: impl Into<String>) -> ContextResult<T>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<AppError>,
{
    fn with_context(self, context: ErrorContext) -> ContextResult<T> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                let app_error = error.into();
                context.log_error(&app_error);
                Err((app_error, context))
            }
        }
    }

    fn with_operation(self, operation: impl Into<String>) -> ContextResult<T> {
        let context = ErrorContext::new(operation);
        self.with_context(context)
    }
}
