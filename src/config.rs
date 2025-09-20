// tokenization-backend/src/config.rs

use serde::{Deserialize, Serialize};
use std::env;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub network: String,
    pub rpc_url: String,
    pub private_key: String,
    pub contract_addresses: ContractAddresses,
    pub gas_limit: u64,
    pub gas_price: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub token_factory: String,
    pub marketplace: String,
    pub compliance: String,
    pub staking: String,
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

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok();


        // Try env-specific file if RUST_ENV is set
        if let Ok(env) = std::env::var("RUST_ENV") {
            let filename = format!(".env.{}", env);
            dotenv::from_filename(&filename).ok();
        }

        let server = ServerConfig {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            max_connections: env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidMaxConnections)?,
            timeout_seconds: env::var("TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTimeout)?,
        };

        let database = DatabaseConfig {
            url: env::var("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?,
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidDbMaxConnections)?,
            min_connections: env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidDbMinConnections)?,
            connection_timeout: env::var("DB_CONNECTION_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidDbConnectionTimeout)?,
            idle_timeout: env::var("DB_IDLE_TIMEOUT")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidDbIdleTimeout)?,
        };

        let jwt = JwtConfig {
            secret: env::var("JWT_SECRET").map_err(|_| ConfigError::MissingJwtSecret)?,
            expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidJwtExpiration)?,
            refresh_expiration_days: env::var("JWT_REFRESH_EXPIRATION_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidJwtRefreshExpiration)?,
            algorithm: env::var("JWT_ALGORITHM").unwrap_or_else(|_| "HS256".to_string()),
        };

        let blockchain = BlockchainConfig {
            network: env::var("BLOCKCHAIN_NETWORK").unwrap_or_else(|_| "localhost".to_string()),
            rpc_url: env::var("BLOCKCHAIN_RPC_URL")
                .map_err(|_| ConfigError::MissingBlockchainRpcUrl)?,
            private_key: env::var("BLOCKCHAIN_PRIVATE_KEY")
                .map_err(|_| ConfigError::MissingBlockchainPrivateKey)?,
            contract_addresses: ContractAddresses {
                token_factory: env::var("CONTRACT_TOKEN_FACTORY")
                    .map_err(|_| ConfigError::MissingTokenFactoryContract)?,
                marketplace: env::var("CONTRACT_MARKETPLACE")
                    .map_err(|_| ConfigError::MissingMarketplaceContract)?,
                compliance: env::var("CONTRACT_COMPLIANCE")
                    .map_err(|_| ConfigError::MissingComplianceContract)?,
                staking: env::var("CONTRACT_STAKING")
                    .map_err(|_| ConfigError::MissingStakingContract)?,
            },
            gas_limit: env::var("BLOCKCHAIN_GAS_LIMIT")
                .unwrap_or_else(|_| "3000000".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidGasLimit)?,
            gas_price: env::var("BLOCKCHAIN_GAS_PRICE")
                .unwrap_or_else(|_| "20000000000".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidGasPrice)?,
        };

        
        dotenv::dotenv().ok();
        println!("Loaded env file, FIREBASE_KEY={:?}", std::env::var("FIREBASE_KEY"));


        let compliance = ComplianceConfig {
            kyc_provider: env::var("KYC_PROVIDER").unwrap_or_else(|_| "jumio".to_string()),
            kyc_api_key: env::var("KYC_API_KEY").map_err(|_| ConfigError::MissingKycApiKey)?,
            aml_provider: env::var("AML_PROVIDER").unwrap_or_else(|_| "chainalysis".to_string()),
            aml_api_key: env::var("AML_API_KEY").map_err(|_| ConfigError::MissingAmlApiKey)?,
            auto_verification: env::var("AUTO_VERIFICATION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidAutoVerification)?,
            verification_timeout_hours: env::var("VERIFICATION_TIMEOUT_HOURS")
                .unwrap_or_else(|_| "72".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidVerificationTimeout)?,
        };

        let notification = NotificationConfig {
            email: EmailConfig {
                smtp_host: env::var("SMTP_HOST").map_err(|_| ConfigError::MissingSmtpHost)?,
                smtp_port: env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()
                    .map_err(|_| ConfigError::InvalidSmtpPort)?,
                smtp_username: env::var("SMTP_USERNAME")
                    .map_err(|_| ConfigError::MissingSmtpUsername)?,
                smtp_password: env::var("SMTP_PASSWORD")
                    .map_err(|_| ConfigError::MissingSmtpPassword)?,
                from_address: env::var("FROM_EMAIL").map_err(|_| ConfigError::MissingFromEmail)?,
                from_name: env::var("FROM_NAME")
                    .unwrap_or_else(|_| "Tokenization Platform".to_string()),
            },
            push: PushConfig {
                firebase_key: env::var("FIREBASE_KEY").ok(),
                apns_key: env::var("APNS_KEY").ok(),
                apns_key_id: env::var("APNS_KEY_ID").ok(),
                apns_team_id: env::var("APNS_TEAM_ID").ok(),
            },
            webhook_url: env::var("WEBHOOK_URL").ok(),
        };


        let security = SecurityConfig {
            bcrypt_cost: env::var("BCRYPT_COST")
                .unwrap_or_else(|_| "12".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidBcryptCost)?,
            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidRateLimitRequests)?,
            rate_limit_window_seconds: env::var("RATE_LIMIT_WINDOW_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidRateLimitWindow)?,
            session_timeout_minutes: env::var("SESSION_TIMEOUT_MINUTES")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidSessionTimeout)?,
            max_login_attempts: env::var("MAX_LOGIN_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidMaxLoginAttempts)?,
            lockout_duration_minutes: env::var("LOCKOUT_DURATION_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidLockoutDuration)?,
        };

        Ok(Config {
            server,
            database,
            jwt,
            blockchain,
            compliance,
            notification,
            security,
        })
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate JWT secret length
        if self.jwt.secret.len() < 32 {
            return Err(ConfigError::JwtSecretTooShort);
        }

        // Validate bcrypt cost
        if self.security.bcrypt_cost < 4 || self.security.bcrypt_cost > 31 {
            return Err(ConfigError::InvalidBcryptCostRange);
        }

        // Validate port range
        if self.server.port < 1024 {
            return Err(ConfigError::InvalidPortRange);
        }

        // Validate database connection pool settings
        if self.database.min_connections > self.database.max_connections {
            return Err(ConfigError::InvalidDbConnectionPool);
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid port number")]
    InvalidPort,
    #[error("Invalid max connections")]
    InvalidMaxConnections,
    #[error("Invalid timeout")]
    InvalidTimeout,
    #[error("Missing database URL")]
    MissingDatabaseUrl,
    #[error("Invalid database max connections")]
    InvalidDbMaxConnections,
    #[error("Invalid database min connections")]
    InvalidDbMinConnections,
    #[error("Invalid database connection timeout")]
    InvalidDbConnectionTimeout,
    #[error("Invalid database idle timeout")]
    InvalidDbIdleTimeout,
    #[error("Missing JWT secret")]
    MissingJwtSecret,
    #[error("Invalid JWT expiration")]
    InvalidJwtExpiration,
    #[error("Invalid JWT refresh expiration")]
    InvalidJwtRefreshExpiration,
    #[error("Missing blockchain RPC URL")]
    MissingBlockchainRpcUrl,
    #[error("Missing blockchain private key")]
    MissingBlockchainPrivateKey,
    #[error("Missing token factory contract address")]
    MissingTokenFactoryContract,
    #[error("Missing marketplace contract address")]
    MissingMarketplaceContract,
    #[error("Missing compliance contract address")]
    MissingComplianceContract,
    #[error("Missing staking contract address")]
    MissingStakingContract,
    #[error("Invalid gas limit")]
    InvalidGasLimit,
    #[error("Invalid gas price")]
    InvalidGasPrice,
    #[error("Missing KYC API key")]
    MissingKycApiKey,
    #[error("Missing AML API key")]
    MissingAmlApiKey,
    #[error("Invalid auto verification setting")]
    InvalidAutoVerification,
    #[error("Invalid verification timeout")]
    InvalidVerificationTimeout,
    #[error("Missing SMTP host")]
    MissingSmtpHost,
    #[error("Invalid SMTP port")]
    InvalidSmtpPort,
    #[error("Missing SMTP username")]
    MissingSmtpUsername,
    #[error("Missing SMTP password")]
    MissingSmtpPassword,
    #[error("Missing from email")]
    MissingFromEmail,
    #[error("Missing Firebase key")]
    MissingFirebaseKey,
    #[error("Missing APNS key")]
    MissingApnsKey,
    #[error("Missing APNS key ID")]
    MissingApnsKeyId,
    #[error("Missing APNS team ID")]
    MissingApnsTeamId,
    #[error("Invalid bcrypt cost")]
    InvalidBcryptCost,
    #[error("Invalid rate limit requests")]
    InvalidRateLimitRequests,
    #[error("Invalid rate limit window")]
    InvalidRateLimitWindow,
    #[error("Invalid session timeout")]
    InvalidSessionTimeout,
    #[error("Invalid max login attempts")]
    InvalidMaxLoginAttempts,
    #[error("Invalid lockout duration")]
    InvalidLockoutDuration,
    #[error("JWT secret must be at least 32 characters")]
    JwtSecretTooShort,
    #[error("Bcrypt cost must be between 4 and 31")]
    InvalidBcryptCostRange,
    #[error("Port must be between 1024 and 65535")]
    InvalidPortRange,
    #[error("Database min connections cannot be greater than max connections")]
    InvalidDbConnectionPool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_origins: vec!["http://localhost:3000".to_string()],
                max_connections: 100,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/tokenization".to_string(),
                max_connections: 10,
                min_connections: 1,
                connection_timeout: 30,
                idle_timeout: 600,
            },
            jwt: JwtConfig {
                secret: "your-super-secret-jwt-key-here-at-least-32-chars".to_string(),
                expiration_hours: 24,
                refresh_expiration_days: 30,
                algorithm: "HS256".to_string(),
            },
            blockchain: BlockchainConfig {
                network: "amoy".to_string(),  // Changed from "localhost"
                rpc_url: "https://rpc-amoy.polygon.technology".to_string(),  // Changed from localhost
                private_key: "0x".to_string(),  // Add your actual private key
                contract_addresses: ContractAddresses {
                    token_factory: "0x".to_string(),  // Add your deployed contract addresses
                    marketplace: "0x".to_string(),
                    compliance: "0x".to_string(),
                    staking: "0x".to_string(),
                },
                gas_limit: 3000000,
                gas_price: 20000000000,  // 20 gwei - might be too high for Amoy
            },
            compliance: ComplianceConfig {
                kyc_provider: "jumio".to_string(),
                kyc_api_key: "".to_string(),
                aml_provider: "chainalysis".to_string(),
                aml_api_key: "".to_string(),
                auto_verification: false,
                verification_timeout_hours: 72,
            },
            // notification: NotificationConfig {
            //     email: EmailConfig {
            //         smtp_host: "smtp.gmail.com".to_string(),
            //         smtp_port: 587,
            //         smtp_username: "".to_string(),
            //         smtp_password: "".to_string(),
            //         from_address: "noreply@tokenization.com".to_string(),
            //         from_name: "Tokenization Platform".to_string(),
            //     },
            //     push: PushConfig {
            //         firebase_key: "".to_string(),
            //         apns_key: "".to_string(),
            //         apns_key_id: "".to_string(),
            //         apns_team_id: "".to_string(),
            //     },
            //     webhook_url: None,
            // },

            notification: NotificationConfig {
                email: EmailConfig {
                    smtp_host: "smtp.gmail.com".to_string(),
                    smtp_port: 587,
                    smtp_username: "".to_string(),
                    smtp_password: "".to_string(),
                    from_address: "noreply@tokenization.com".to_string(),
                    from_name: "Tokenization Platform".to_string(),
                },
                push: PushConfig {
                    firebase_key: None,
                    apns_key: None,
                    apns_key_id: None,
                    apns_team_id: None,
                },
                webhook_url: None,
            },

            security: SecurityConfig {
                bcrypt_cost: 12,
                rate_limit_requests: 100,
                rate_limit_window_seconds: 3600,
                session_timeout_minutes: 30,
                max_login_attempts: 5,
                lockout_duration_minutes: 15,
            },
        }
    }
}

