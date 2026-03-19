// src/config.rs

use serde::{Deserialize, Serialize};
use std::env;
use ethers::types::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub blockchain: BlockchainConfig,
    pub compliance: ComplianceConfig,
    pub notification: NotificationConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: u64,
    pub refresh_expiration_days: u64,
    pub algorithm: String,
}

// ===== FIX #1: FLATTEN THE STRUCT AND USE ETHERS::TYPES::ADDRESS =====
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub network: String,
    pub rpc_url: String,
    pub deployer_private_key: String, // Renamed from private_key for clarity
    pub gas_limit: u64,
    pub gas_price: u64,
    // Direct contract addresses
    pub token_factory_proxy_address: Address,
    pub marketplace_core_proxy_address: Address,
    pub compliance_manager_proxy_address: Address,
    pub hybrid_asset_tokenizer_proxy_address: Address,
    // Add any other addresses you need here
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub kyc_provider: String,
    pub kyc_api_key: String,
    pub aml_provider: String,
    pub aml_api_key: String,
    pub auto_verification: bool,
    pub verification_timeout_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub email: EmailConfig,
    pub push: PushConfig,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    pub from_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushConfig {
    pub firebase_key: Option<String>,
    pub apns_key: Option<String>,
    pub apns_key_id: Option<String>,
    pub apns_team_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub bcrypt_cost: u32,
    pub rate_limit_requests: u32,
    pub rate_limit_window_seconds: u64,
    pub session_timeout_minutes: u64,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingVar(String),
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok();
        if let Ok(env) = std::env::var("RUST_ENV") {
            dotenv::from_filename(format!(".env.{}", env)).ok();
        }

        // Helper to load and parse a variable
        fn get_var<T>(name: &str) -> Result<T, ConfigError>
        where
            T: std::str::FromStr,
            T::Err: std::fmt::Display,
        {
            env::var(name)
                .map_err(|_| ConfigError::MissingVar(name.to_string()))?
                .parse::<T>()
                .map_err(|e| ConfigError::InvalidValue(name.to_string(), e.to_string()))
        }

        // Helper for optional variables
        fn get_opt_var<T>(name: &str) -> Result<Option<T>, ConfigError>
        where
            T: std::str::FromStr,
            T::Err: std::fmt::Display,
        {
            match env::var(name) {
                Ok(val) => val.parse::<T>().map(Some).map_err(|e| ConfigError::InvalidValue(name.to_string(), e.to_string())),
                Err(env::VarError::NotPresent) => Ok(None),
                Err(e) => Err(ConfigError::InvalidValue(name.to_string(), e.to_string())),
            }
        }

        let server = ServerConfig {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: get_var("SERVER_PORT")?,
            cors_origins: env::var("CORS_ORIGINS").unwrap_or_default().split(',').map(String::from).collect(),
            max_connections: get_var("MAX_CONNECTIONS")?,
            timeout_seconds: get_var("TIMEOUT_SECONDS")?,
        };

        let database = DatabaseConfig {
            url: get_var("DATABASE_URL")?,
            max_connections: get_var("DB_MAX_CONNECTIONS")?,
            min_connections: get_var("DB_MIN_CONNECTIONS")?,
            connection_timeout: get_var("DB_CONNECTION_TIMEOUT")?,
            idle_timeout: get_var("DB_IDLE_TIMEOUT")?,
        };

        let jwt = JwtConfig {
            secret: get_var("JWT_SECRET")?,
            expiration_hours: get_var("JWT_EXPIRATION_HOURS")?,
            refresh_expiration_days: get_var("JWT_REFRESH_EXPIRATION_DAYS")?,
            algorithm: env::var("JWT_ALGORITHM").unwrap_or_else(|_| "HS256".to_string()),
        };

        // ===== FIX #3: LOAD THE NEW BLOCKCHAIN VARIABLES FROM .ENV =====
        let blockchain = BlockchainConfig {
            network: env::var("BLOCKCHAIN_NETWORK").unwrap_or_else(|_| "amoy".to_string()),
            rpc_url: get_var("BLOCKCHAIN_RPC_URL")?,
            deployer_private_key: get_var("DEPLOYER_PRIVATE_KEY")?,
            gas_limit: get_var("BLOCKCHAIN_GAS_LIMIT")?,
            gas_price: get_var("BLOCKCHAIN_GAS_PRICE")?,
            token_factory_proxy_address: get_var("TOKEN_FACTORY_PROXY_ADDRESS")?,
            marketplace_core_proxy_address: get_var("MARKETPLACE_CORE_PROXY_ADDRESS")?,
            compliance_manager_proxy_address: get_var("COMPLIANCE_MANAGER_PROXY_ADDRESS")?,
            hybrid_asset_tokenizer_proxy_address: get_var("HYBRID_ASSET_TOKENIZER_PROXY_ADDRESS")?,
        };

        let compliance = ComplianceConfig {
            kyc_provider: env::var("KYC_PROVIDER").unwrap_or_else(|_| "internal".to_string()),
            kyc_api_key: get_var("KYC_API_KEY")?,
            aml_provider: env::var("AML_PROVIDER").unwrap_or_else(|_| "internal".to_string()),
            aml_api_key: get_var("AML_API_KEY")?,
            auto_verification: get_var("AUTO_VERIFICATION")?,
            verification_timeout_hours: get_var("VERIFICATION_TIMEOUT_HOURS")?,
        };

        let notification = NotificationConfig {
            email: EmailConfig {
                smtp_host: get_var("SMTP_HOST")?,
                smtp_port: get_var("SMTP_PORT")?,
                smtp_username: get_var("SMTP_USERNAME")?,
                smtp_password: get_var("SMTP_PASSWORD")?,
                from_address: get_var("FROM_EMAIL")?,
                from_name: env::var("FROM_NAME").unwrap_or_else(|_| "Tokenization Platform".to_string()),
            },
            push: PushConfig {
                firebase_key: get_opt_var("FIREBASE_KEY")?,
                apns_key: get_opt_var("APNS_KEY")?,
                apns_key_id: get_opt_var("APNS_KEY_ID")?,
                apns_team_id: get_opt_var("APNS_TEAM_ID")?,
            },
            webhook_url: env::var("WEBHOOK_URL").ok(),
        };

        let security = SecurityConfig {
            bcrypt_cost: get_var("BCRYPT_COST")?,
            rate_limit_requests: get_var("RATE_LIMIT_REQUESTS")?,
            rate_limit_window_seconds: get_var("RATE_LIMIT_WINDOW_SECONDS")?,
            session_timeout_minutes: get_var("SESSION_TIMEOUT_MINUTES")?,
            max_login_attempts: get_var("MAX_LOGIN_ATTEMPTS")?,
            lockout_duration_minutes: get_var("LOCKOUT_DURATION_MINUTES")?,
        };

        Ok(Config { server, database, jwt, blockchain, compliance, notification, security })
    }
}