// // src/models/user.rs

// use chrono::{DateTime, Utc};
// use serde::{Deserialize, Serialize};
// use sqlx::FromRow;
// use uuid::Uuid;
// use tracing_subscriber::fmt;
// use crate::Result;
// use std::str::FromStr;
// use std::fmt::Display;


// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
// #[sqlx(type_name = "user_role", rename_all = "lowercase")]
// pub enum UserRole {
//     Admin,
//     User,
//     Investor,
//     ProjectManager,
//     ComplianceOfficer,
//     Moderator,
//     Developer,
// }


// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
// #[sqlx(type_name = "user_status", rename_all = "snake_case")]
// pub enum UserStatus {
//     Active,
//     Inactive,
//     Suspended,
//     PendingVerification,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
// pub struct User {
//     pub id: Uuid,
//     pub email: String,
//     pub password_hash: String,
//     pub first_name: Option<String>,
//     pub last_name: Option<String>,
//     pub phone: Option<String>,
//     pub date_of_birth: Option<chrono::NaiveDate>,
//     pub nationality: Option<String>,
//     pub address: Option<serde_json::Value>,
//     pub wallet_address: Option<String>,
//     pub username: Option<String>,
//     pub role: UserRole,
//     pub status: UserStatus,
//     pub email_verified: Option<bool>,
//     pub phone_verified: Option<bool>,
//     pub two_factor_enabled: Option<bool>,
//     pub two_factor_secret: Option<String>,
//     pub last_login: Option<DateTime<Utc>>,
//     pub login_attempts: Option<i32>,
//     pub locked_until: Option<DateTime<Utc>>,
//     pub reset_token: Option<String>,
//     pub reset_token_expires: Option<DateTime<Utc>>,
//     pub verification_token: Option<String>,
//     pub verification_token_expires: Option<DateTime<Utc>>,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: DateTime<Utc>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct CreateUserRequest {
//     pub email: String,
//     pub password: String,
//     pub first_name: Option<String>,
//     pub last_name: Option<String>,
//     pub phone: Option<String>,
//     pub nationality: Option<String>,
//     pub date_of_birth: Option<chrono::NaiveDate>,
//     pub address: Option<serde_json::Value>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct LoginRequest {
//     pub email: String,
//     pub password: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct LoginResponse {
//     pub user: UserResponse,
//     pub token: String,
//     pub expires_at: DateTime<Utc>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct UserResponse {
//     pub id: Uuid,
//     pub email: String,
//     pub first_name: Option<String>,
//     pub last_name: Option<String>,
//     pub phone: Option<String>,
//     pub nationality: Option<String>,
//     pub date_of_birth: Option<chrono::NaiveDate>,
//     pub wallet_address: Option<String>,
//     pub username: Option<String>,
//     pub role: UserRole,
//     pub status: UserStatus,
//     pub email_verified: Option<bool>,
//     pub phone_verified: Option<bool>,
//     pub two_factor_enabled: Option<bool>,
//     pub last_login: Option<DateTime<Utc>>,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: DateTime<Utc>,
// }

// impl From<User> for UserResponse {
//     fn from(user: User) -> Self {
//         Self {
//             id: user.id,
//             email: user.email,
//             first_name: user.first_name,
//             last_name: user.last_name,
//             phone: user.phone,
//             nationality: user.nationality,
//             date_of_birth: user.date_of_birth,
//             wallet_address: None, // TODO: Enable after wallet_address migration is applied
//             username: user.username,
//             role: user.role,
//             status: user.status,
//             email_verified: user.email_verified,
//             phone_verified: user.phone_verified,
//             two_factor_enabled: user.two_factor_enabled,
//             last_login: user.last_login,
//             created_at: user.created_at,
//             updated_at: user.updated_at,
//         }
//     }
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct UpdateUserRequest {
//     pub first_name: Option<String>,
//     pub last_name: Option<String>,
//     pub phone: Option<String>,
//     pub nationality: Option<String>,
//     pub date_of_birth: Option<chrono::NaiveDate>,
//     pub address: Option<serde_json::Value>,
//     pub username: Option<String>,
// }





// // Display implementations for enums
// impl Display for UserRole {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result {
//         let s = match self {
//             UserRole::Admin => "admin",
//             UserRole::User => "user",
//             UserRole::Investor => "investor",
//             UserRole::ProjectManager => "project_manager",
//             UserRole::ComplianceOfficer => "compliance_officer",
//             UserRole::Moderator => "moderator",
//             UserRole::Developer => "developer",
//         };
//         write!(f, "{}", s)
//     }
// }

// impl Display for UserStatus {
//      fn fmt(&self, f: &mut fmt::Formatter) -> Result {
//         let s = match self {
//             UserStatus::Active => "active",
//             UserStatus::Inactive => "inactive",
//             UserStatus::Suspended => "suspended",
//             UserStatus::PendingVerification => "pending_verification",
//         };
//         write!(f, "{}", s)
//     }
// }


// src/models/user.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
    Investor,
    ProjectManager,
    ComplianceOfficer,
    Moderator,
    Developer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub nationality: Option<String>,
    pub address: Option<serde_json::Value>,
    pub wallet_address: Option<String>,
    pub username: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub email_verified: Option<bool>,
    pub phone_verified: Option<bool>,
    pub two_factor_enabled: Option<bool>,
    pub two_factor_secret: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub login_attempts: Option<i32>,
    pub locked_until: Option<DateTime<Utc>>,
    pub reset_token: Option<String>,
    pub reset_token_expires: Option<DateTime<Utc>>,
    pub verification_token: Option<String>,
    pub verification_token_expires: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub nationality: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub address: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub nationality: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub wallet_address: Option<String>,
    pub username: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub email_verified: Option<bool>,
    pub phone_verified: Option<bool>,
    pub two_factor_enabled: Option<bool>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            phone: user.phone,
            nationality: user.nationality,
            date_of_birth: user.date_of_birth,
            wallet_address: None, // TODO: Enable after wallet_address migration is applied
            username: user.username,
            role: user.role,
            status: user.status,
            email_verified: user.email_verified,
            phone_verified: user.phone_verified,
            two_factor_enabled: user.two_factor_enabled,
            last_login: user.last_login,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub nationality: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub address: Option<serde_json::Value>,
    pub username: Option<String>,
}

// FromStr implementation for UserRole
impl FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "user" => Ok(UserRole::User),
            "investor" => Ok(UserRole::Investor),
            "projectmanager" | "project_manager" => Ok(UserRole::ProjectManager),
            "complianceofficer" | "compliance_officer" => Ok(UserRole::ComplianceOfficer),
            "moderator" => Ok(UserRole::Moderator),
            "developer" => Ok(UserRole::Developer),
            _ => Err(format!("Unknown user role: {}", s)),
        }
    }
}

// FromStr implementation for UserStatus
impl FromStr for UserStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(UserStatus::Active),
            "inactive" => Ok(UserStatus::Inactive),
            "suspended" => Ok(UserStatus::Suspended),
            "pending_verification" | "pendingverification" => Ok(UserStatus::PendingVerification),
            _ => Err(format!("Unknown user status: {}", s)),
        }
    }
}