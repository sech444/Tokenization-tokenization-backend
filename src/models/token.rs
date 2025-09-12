

// // src/models/token.rs

// use chrono::{DateTime, Utc};
// use serde::{Deserialize, Serialize};
// use sqlx::FromRow;
// use uuid::Uuid;
// use validator::Validate;

// #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
// pub struct Token {
//     pub id: Uuid,
//     pub project_id: Uuid,
//     pub name: String,
//     pub symbol: String,
//     pub description: Option<String>,
//     pub token_type: TokenType,
//     pub total_supply: i64,
//     pub circulating_supply: Option<i64>,
//     pub decimals: Option<i32>,
//     pub owner_id: Uuid,
//     pub metadata: Option<serde_json::Value>,
//     pub is_active: bool,
//     pub current_price: i64,  // in cents
//     pub initial_price: i64,  // in cents
//     pub contract_address: String,
//     pub status: TokenStatus,
//     pub metadata_uri: Option<String>,
//     pub compliance_rules: serde_json::Value,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: DateTime<Utc>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
// #[sqlx(type_name = "token_status", rename_all = "lowercase")]
// pub enum TokenStatus {
//     Pending,
//     Active,
//     Paused,
//     Cancelled,
//     Completed,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
// #[sqlx(type_name = "token_type", rename_all = "snake_case")]
// pub enum TokenType {
//     Fungible,
//     #[sqlx(rename = "non_fungible")]
//     NonFungible,
//     #[sqlx(rename = "semi_fungible")]
//     SemiFungible,
//     Utility,
//     Security,
//     Governance,
// }

// #[derive(Debug, Deserialize, Validate)]
// pub struct CreateTokenRequest {
//     pub project_id: Uuid,
//     pub owner_id: Uuid,

//     #[validate(length(min = 1, max = 100))]
//     pub name: String,

//     #[validate(length(min = 1, max = 10))]
//     pub symbol: String,

//     #[validate(length(max = 500))]
//     pub description: Option<String>,

//     pub token_type: TokenType,

//     #[validate(range(min = 1))]
//     pub total_supply: i64,

//     #[validate(range(min = 1))]
//     pub initial_price: i64, // in cents

//     pub contract_address: String,

//     #[validate(range(min = 0, max = 18))]
//     pub decimals: Option<i32>,

//     pub metadata: Option<serde_json::Value>,
//     pub metadata_uri: Option<String>,
//     pub compliance_rules: Option<serde_json::Value>,
// }

// #[derive(Debug, Deserialize, Validate)]
// pub struct UpdateTokenRequest {
//     #[validate(length(min = 1, max = 100))]
//     pub name: Option<String>,

//     #[validate(length(max = 500))]
//     pub description: Option<String>,

//     pub metadata: Option<serde_json::Value>,
//     pub metadata_uri: Option<String>,
//     pub compliance_rules: Option<serde_json::Value>,
//     pub is_active: Option<bool>,
//     pub status: Option<TokenStatus>,
// }

// #[derive(Debug, Deserialize, Validate)]
// pub struct MintTokenRequest {
//     #[validate(length(min = 1))]
//     pub to_address: String,

//     #[validate(range(min = 1))]
//     pub amount: i64,

//     pub metadata: Option<serde_json::Value>,
//     pub reason: Option<String>,
// }

// #[derive(Debug, Deserialize, Validate)]
// pub struct BurnTokenRequest {
//     pub from_address: Option<String>, // If None, burns from token owner

//     #[validate(range(min = 1))]
//     pub amount: i64,

//     pub reason: Option<String>,
// }

// // Add these aliases for backward compatibility
// pub type MintRequest = MintTokenRequest;
// pub type BurnRequest = BurnTokenRequest;

// #[derive(Debug, Deserialize, Validate)]
// pub struct TransferTokenRequest {
//     #[validate(length(min = 1))]
//     pub from_address: String,

//     #[validate(length(min = 1))]
//     pub to_address: String,

//     #[validate(range(min = 1))]
//     pub amount: i64,

//     pub metadata: Option<serde_json::Value>,
// }

// // Response DTOs
// #[derive(Debug, Serialize)]
// pub struct TokenResponse {
//     pub id: Uuid,
//     pub project_id: Uuid,
//     pub name: String,
//     pub symbol: String,
//     pub description: Option<String>,
//     pub token_type: TokenType,
//     pub total_supply: i64,
//     pub circulating_supply: Option<i64>,
//     pub decimals: Option<i32>,
//     pub owner_id: Uuid,
//     pub metadata: Option<serde_json::Value>,
//     pub is_active: bool,
//     pub current_price: i64,
//     pub initial_price: i64,
//     pub contract_address: String,
//     pub status: TokenStatus,
//     pub metadata_uri: Option<String>,
//     pub compliance_rules: serde_json::Value,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: DateTime<Utc>,
// }

// impl From<Token> for TokenResponse {
//     fn from(token: Token) -> Self {
//         Self {
//             id: token.id,
//             project_id: token.project_id,
//             name: token.name,
//             symbol: token.symbol,
//             description: token.description,
//             token_type: token.token_type,
//             total_supply: token.total_supply,
//             circulating_supply: token.circulating_supply,
//             decimals: token.decimals,
//             owner_id: token.owner_id,
//             metadata: token.metadata,
//             is_active: token.is_active,
//             current_price: token.current_price,
//             initial_price: token.initial_price,
//             contract_address: token.contract_address,
//             status: token.status,
//             metadata_uri: token.metadata_uri,
//             compliance_rules: token.compliance_rules,
//             created_at: token.created_at,
//             updated_at: token.updated_at,
//         }
//     }
// }


// src/models/token.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;
use std::fmt;
use std::str::FromStr;
use crate::models::UserRole;
use crate::models::UserStatus;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Token {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub token_type: TokenType,
    pub total_supply: i64,
    pub circulating_supply: Option<i64>,
    pub decimals: Option<i32>,
    pub owner_id: Uuid,
    pub metadata: Option<serde_json::Value>,
    pub is_active: bool,
    pub current_price: i64,  // in cents
    pub initial_price: i64,  // in cents
    pub contract_address: String,
    pub status: TokenStatus,
    pub metadata_uri: Option<String>,
    pub compliance_rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "token_status", rename_all = "lowercase")]
pub enum TokenStatus {
    Pending,
    Active,
    Paused,
    Cancelled,
    Completed,
}

impl fmt::Display for TokenStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TokenStatus::Pending => "pending",
            TokenStatus::Active => "active",
            TokenStatus::Paused => "paused",
            TokenStatus::Cancelled => "cancelled",
            TokenStatus::Completed => "completed",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for TokenStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TokenStatus::Pending),
            "active" => Ok(TokenStatus::Active),
            "paused" => Ok(TokenStatus::Paused),
            "cancelled" => Ok(TokenStatus::Cancelled),
            "completed" => Ok(TokenStatus::Completed),
            _ => Err(format!("Invalid token status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "token_type", rename_all = "snake_case")]
pub enum TokenType {
    Fungible,
    #[sqlx(rename = "non_fungible")]
    NonFungible,
    #[sqlx(rename = "semi_fungible")]
    SemiFungible,
    Utility,
    Security,
    Governance,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TokenType::Fungible => "fungible",
            TokenType::NonFungible => "non_fungible",
            TokenType::SemiFungible => "semi_fungible",
            TokenType::Utility => "utility",
            TokenType::Security => "security",
            TokenType::Governance => "governance",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for TokenType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fungible" => Ok(TokenType::Fungible),
            "non_fungible" => Ok(TokenType::NonFungible),
            "semi_fungible" => Ok(TokenType::SemiFungible),
            "utility" => Ok(TokenType::Utility),
            "security" => Ok(TokenType::Security),
            "governance" => Ok(TokenType::Governance),
            _ => Err(format!("Invalid token type: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTokenRequest {
    pub project_id: Uuid,
    pub owner_id: Uuid,

    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(min = 1, max = 10))]
    pub symbol: String,

    #[validate(length(max = 500))]
    pub description: Option<String>,

    pub token_type: TokenType,

    #[validate(range(min = 1))]
    pub total_supply: i64,

    #[validate(range(min = 1))]
    pub initial_price: i64, // in cents

    pub contract_address: String,

    #[validate(range(min = 0, max = 18))]
    pub decimals: Option<i32>,

    pub metadata: Option<serde_json::Value>,
    pub metadata_uri: Option<String>,
    pub compliance_rules: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTokenRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    #[validate(length(max = 500))]
    pub description: Option<String>,

    pub metadata: Option<serde_json::Value>,
    pub metadata_uri: Option<String>,
    pub compliance_rules: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub status: Option<TokenStatus>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct MintTokenRequest {
    #[validate(length(min = 1))]
    pub to_address: String,

    #[validate(range(min = 1))]
    pub amount: i64,

    pub metadata: Option<serde_json::Value>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct BurnTokenRequest {
    pub from_address: Option<String>, // If None, burns from token owner

    #[validate(range(min = 1))]
    pub amount: i64,

    pub reason: Option<String>,
}

// Add these aliases for backward compatibility
pub type MintRequest = MintTokenRequest;
pub type BurnRequest = BurnTokenRequest;

#[derive(Debug, Deserialize, Validate)]
pub struct TransferTokenRequest {
    #[validate(length(min = 1))]
    pub from_address: String,

    #[validate(length(min = 1))]
    pub to_address: String,

    #[validate(range(min = 1))]
    pub amount: i64,

    pub metadata: Option<serde_json::Value>,
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub token_type: TokenType,
    pub total_supply: i64,
    pub circulating_supply: Option<i64>,
    pub decimals: Option<i32>,
    pub owner_id: Uuid,
    pub metadata: Option<serde_json::Value>,
    pub is_active: bool,
    pub current_price: i64,
    pub initial_price: i64,
    pub contract_address: String,
    pub status: TokenStatus,
    pub metadata_uri: Option<String>,
    pub compliance_rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Token> for TokenResponse {
    fn from(token: Token) -> Self {
        Self {
            id: token.id,
            project_id: token.project_id,
            name: token.name,
            symbol: token.symbol,
            description: token.description,
            token_type: token.token_type,
            total_supply: token.total_supply,
            circulating_supply: token.circulating_supply,
            decimals: token.decimals,
            owner_id: token.owner_id,
            metadata: token.metadata,
            is_active: token.is_active,
            current_price: token.current_price,
            initial_price: token.initial_price,
            contract_address: token.contract_address,
            status: token.status,
            metadata_uri: token.metadata_uri,
            compliance_rules: token.compliance_rules,
            created_at: token.created_at,
            updated_at: token.updated_at,
        }
    }
}
// Add these to your src/models/user.rs file (after the existing structs)



impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
            UserRole::Investor => write!(f, "investor"),
            UserRole::ProjectManager => write!(f, "projectmanager"),
            UserRole::ComplianceOfficer => write!(f, "complianceofficer"),
            UserRole::Moderator => write!(f, "moderator"),
            UserRole::Developer => write!(f, "developer"),
        }
    }
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserStatus::Active => write!(f, "active"),
            UserStatus::Inactive => write!(f, "inactive"),
            UserStatus::Suspended => write!(f, "suspended"),
            UserStatus::PendingVerification => write!(f, "pending_verification"),
        }
    }
}