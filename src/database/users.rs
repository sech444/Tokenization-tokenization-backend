

// // src/database/users.rs

// use sqlx::{PgPool, Row};
// use uuid::Uuid;
// use std::fmt;
// use std::str::FromStr;
// use chrono::{DateTime, Utc};
// use crate::models::{User, UserRole, UserStatus};
// use crate::utils::errors::AppError;

// // Add Display trait for UserRole
// impl fmt::Display for UserRole {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             UserRole::Admin => write!(f, "admin"),
//             UserRole::User => write!(f, "user"),
//             UserRole::Investor => write!(f, "investor"),
//             UserRole::ProjectManager => write!(f, "projectmanager"),
//             UserRole::ComplianceOfficer => write!(f, "complianceofficer"),
//             UserRole::Moderator => write!(f, "moderator"),
//             UserRole::Developer => write!(f, "developer"),
//         }
//     }
// }

// // Add FromStr trait for UserRole
// impl FromStr for UserRole {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "admin" => Ok(UserRole::Admin),
//             "user" => Ok(UserRole::User),
//             "investor" => Ok(UserRole::Investor),
//             "projectmanager" => Ok(UserRole::ProjectManager),
//             "complianceofficer" => Ok(UserRole::ComplianceOfficer),
//             "moderator" => Ok(UserRole::Moderator),
//             "developer" => Ok(UserRole::Developer),
//             _ => Err(format!("Unknown user role: {}", s)),
//         }
//     }
// }

// // Add Display trait for UserStatus
// impl fmt::Display for UserStatus {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             UserStatus::Active => write!(f, "active"),
//             UserStatus::Inactive => write!(f, "inactive"),
//             UserStatus::Suspended => write!(f, "suspended"),
//             UserStatus::PendingVerification => write!(f, "pending_verification"),
//         }
//     }
// }

// // Add FromStr trait for UserStatus
// impl FromStr for UserStatus {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "active" => Ok(UserStatus::Active),
//             "inactive" => Ok(UserStatus::Inactive),
//             "suspended" => Ok(UserStatus::Suspended),
//             "pending_verification" => Ok(UserStatus::PendingVerification),
//             _ => Err(format!("Unknown user status: {}", s)),
//         }
//     }
// }


// fn row_to_user(row: &sqlx::postgres::PgRow) -> Result<User, AppError> {
//     let role_str: String = row.get("role");
//     let role = role_str.parse::<UserRole>()
//         .map_err(|_| AppError::ValidationError("Invalid user role".to_string()))?;
    
//     // Direct enum extraction from SQLx - no need for string conversion
//     let status: UserStatus = row.try_get("status")
//         .map_err(|e| AppError::DatabaseError(format!("Failed to get status: {}", e)))?;

//     Ok(User {
//         id: row.get("id"),
//         email: row.get("email"),
//         password_hash: row.get("password_hash"),
//         first_name: row.get("first_name"),
//         last_name: row.get("last_name"),
//         phone: row.get("phone"),
//         date_of_birth: row.get("date_of_birth"),
//         nationality: row.get("nationality"),
//         address: row.get("address"),
//         wallet_address: row.get("wallet_address"),
//         username: row.get("username"),
//         role,
//         status,
//         email_verified: row.get("email_verified"),
//         phone_verified: row.get("phone_verified"),
//         two_factor_enabled: row.get("two_factor_enabled"),
//         two_factor_secret: row.get("two_factor_secret"),
//         last_login: row.get("last_login"),
//         login_attempts: row.get("login_attempts"),
//         locked_until: row.get("locked_until"),
//         reset_token: row.get("reset_token"),
//         reset_token_expires: row.get("reset_token_expires"),
//         verification_token: row.get("verification_token"),
//         verification_token_expires: row.get("verification_token_expires"),
//         created_at: row.get("created_at"),
//         updated_at: row.get("updated_at"),
//     })
// }


// /// Create a new user in the database
// pub async fn create_user(db: &PgPool, user: &User) -> Result<User, AppError> {
//     let row = sqlx::query(
//         r#"
//         INSERT INTO users (
//             id, email, password_hash, first_name, last_name, phone, date_of_birth, 
//             nationality, address, wallet_address, username, role, status, email_verified, 
//             phone_verified, two_factor_enabled, two_factor_secret, verification_token,
//             verification_token_expires, created_at, updated_at
//         )
//         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::user_role, $13::user_status, $14, $15, $16, $17, $18, $19, $20, $21)
//         RETURNING 
//             id, email, password_hash, first_name, last_name, phone, date_of_birth, 
//             nationality, address, wallet_address, username, role, status, email_verified, 
//             phone_verified, two_factor_enabled, two_factor_secret, last_login, 
//             login_attempts, locked_until, reset_token, reset_token_expires, 
//             verification_token, verification_token_expires, created_at, updated_at
//         "#
//     )
//     .bind(&user.id)
//     .bind(&user.email)
//     .bind(&user.password_hash)
//     .bind(&user.first_name)
//     .bind(&user.last_name)
//     .bind(&user.phone)
//     .bind(&user.date_of_birth)
//     .bind(&user.nationality)
//     .bind(&user.address)
//     .bind(&user.wallet_address)
//     .bind(&user.username)
//     .bind(&user.role)  // Direct binding
//     .bind(&user.status)
//     .bind(&user.email_verified)
//     .bind(&user.phone_verified)
//     .bind(&user.two_factor_enabled)
//     .bind(&user.two_factor_secret)
//     .bind(&user.verification_token)
//     .bind(&user.verification_token_expires)
//     .bind(&user.created_at)
//     .bind(&user.updated_at)  // This is parameter $21
//     .fetch_one(db)
//     .await
//     .map_err(|e| AppError::DatabaseError(e.to_string()))?;

//     row_to_user(&row)
// }
// /// Get user by ID
// pub async fn get_user_by_id(db: &PgPool, id: &Uuid) -> Result<Option<User>, AppError> {
//     let row = sqlx::query(
//         r#"
//         SELECT 
//             id, email, password_hash, first_name, last_name, phone, date_of_birth, 
//             nationality, address, wallet_address,  username, role::text as role, status, email_verified, 
//             phone_verified, two_factor_enabled, two_factor_secret, last_login, 
//             login_attempts, locked_until, reset_token, reset_token_expires, 
//             verification_token, verification_token_expires, created_at, updated_at
//         FROM users WHERE id = $1
//         "#
//     )
//     .bind(id)
//     .fetch_optional(db)
//     .await
//     .map_err(|e| AppError::DatabaseError(e.to_string()))?;

//     match row {
//         Some(r) => Ok(Some(row_to_user(&r)?)),
//         None => Ok(None),
//     }
// }

// /// Get user by email
// pub async fn get_user_by_email(db: &PgPool, email: &str) -> Result<Option<User>, AppError> {
//     let row = sqlx::query(
//         r#"
//         SELECT 
//             id, email, password_hash, first_name, last_name, phone, date_of_birth, 
//             nationality, address, wallet_address,  username, role::text as role, status, email_verified, 
//             phone_verified, two_factor_enabled, two_factor_secret, last_login, 
//             login_attempts, locked_until, reset_token, reset_token_expires, 
//             verification_token, verification_token_expires, created_at, updated_at
//         FROM users WHERE LOWER(email) = LOWER($1)
//         "#
//     )
//     .bind(email)
//     .fetch_optional(db)
//     .await
//     .map_err(|e| AppError::DatabaseError(e.to_string()))?;

//     match row {
//         Some(r) => Ok(Some(row_to_user(&r)?)),
//         None => Ok(None),
//     }
// }



// /// Update user password
// pub async fn update_user_password(db: &PgPool, user_id: Uuid, new_password_hash: &str) -> Result<(), AppError> {
//     sqlx::query(
//         r#"
//         UPDATE users 
//         SET password_hash = $1, updated_at = $2
//         WHERE id = $3
//         "#
//     )
//     .bind(new_password_hash)
//     .bind(Utc::now())
//     .bind(user_id)
//     .execute(db)
//     .await
//     .map_err(|e| AppError::DatabaseError(e.to_string()))?;

//     Ok(())
// }

// /// Update last login timestamp
// pub async fn update_last_login(db: &PgPool, user_id: Uuid) -> Result<(), AppError> {
//     sqlx::query(
//         r#"
//         UPDATE users 
//         SET last_login = $1, updated_at = $2, login_attempts = 0
//         WHERE id = $3
//         "#
//     )
//     .bind(Utc::now())
//     .bind(Utc::now())
//     .bind(user_id)
//     .execute(db)
//     .await
//     .map_err(|e| AppError::DatabaseError(e.to_string()))?;

//     Ok(())
// }

// /// Increment login attempts
// pub async fn increment_login_attempts(db: &PgPool, user_id: Uuid) -> Result<(), AppError> {
//     sqlx::query(
//         r#"
//         UPDATE users 
//         SET login_attempts = login_attempts + 1, updated_at = $1
//         WHERE id = $2
//         "#
//     )
//     .bind(Utc::now())
//     .bind(user_id)
//     .execute(db)
//     .await
//     .map_err(|e| AppError::DatabaseError(e.to_string()))?;

//     Ok(())
// }

// /// Lock user account until specified time
// pub async fn lock_user_account(db: &PgPool, user_id: Uuid, lock_until: DateTime<Utc>) -> Result<(), AppError> {
//     sqlx::query(
//         r#"
//         UPDATE users 
//         SET locked_until = $1, updated_at = $2
//         WHERE id = $3
//         "#
//     )
//     .bind(lock_until)
//     .bind(Utc::now())
//     .bind(user_id)
//     .execute(db)
//     .await
//     .map_err(|e| AppError::DatabaseError(e.to_string()))?;

//     Ok(())
// }

// src/database/users.rs

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::{User, UserRole, UserStatus};
use crate::utils::errors::AppError;

fn row_to_user(row: &sqlx::postgres::PgRow) -> Result<User, AppError> {
    // Extract enums directly - no string conversion needed
    let role: UserRole = row.try_get("role")
        .map_err(|e| AppError::DatabaseError(format!("Failed to get role: {}", e)))?;
    
    let status: UserStatus = row.try_get("status")
        .map_err(|e| AppError::DatabaseError(format!("Failed to get status: {}", e)))?;

    Ok(User {
        id: row.get("id"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        phone: row.get("phone"),
        date_of_birth: row.get("date_of_birth"),
        nationality: row.get("nationality"),
        address: row.get("address"),
        wallet_address: row.get("wallet_address"),
        username: row.get("username"),
        role,
        status,
        email_verified: row.get("email_verified"),
        phone_verified: row.get("phone_verified"),
        two_factor_enabled: row.get("two_factor_enabled"),
        two_factor_secret: row.get("two_factor_secret"),
        last_login: row.get("last_login"),
        login_attempts: row.get("login_attempts"),
        locked_until: row.get("locked_until"),
        reset_token: row.get("reset_token"),
        reset_token_expires: row.get("reset_token_expires"),
        verification_token: row.get("verification_token"),
        verification_token_expires: row.get("verification_token_expires"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

/// Create a new user in the database
pub async fn create_user(db: &PgPool, user: &User) -> Result<User, AppError> {
    let row = sqlx::query(
        r#"
        INSERT INTO users (
            id, email, password_hash, first_name, last_name, phone, date_of_birth, 
            nationality, address, wallet_address, username, role, status, email_verified, 
            phone_verified, two_factor_enabled, two_factor_secret, verification_token,
            verification_token_expires, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
        RETURNING 
            id, email, password_hash, first_name, last_name, phone, date_of_birth, 
            nationality, address, wallet_address, username, role, status, email_verified, 
            phone_verified, two_factor_enabled, two_factor_secret, last_login, 
            login_attempts, locked_until, reset_token, reset_token_expires, 
            verification_token, verification_token_expires, created_at, updated_at
        "#
    )
    .bind(&user.id)
    .bind(&user.email)
    .bind(&user.password_hash)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(&user.phone)
    .bind(&user.date_of_birth)
    .bind(&user.nationality)
    .bind(&user.address)
    .bind(&user.wallet_address)
    .bind(&user.username)
    .bind(&user.role)
    .bind(&user.status)
    .bind(&user.email_verified)
    .bind(&user.phone_verified)
    .bind(&user.two_factor_enabled)
    .bind(&user.two_factor_secret)
    .bind(&user.verification_token)
    .bind(&user.verification_token_expires)
    .bind(&user.created_at)
    .bind(&user.updated_at)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    row_to_user(&row)
}

/// Get user by ID
pub async fn get_user_by_id(db: &PgPool, id: &Uuid) -> Result<Option<User>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            id, email, password_hash, first_name, last_name, phone, date_of_birth, 
            nationality, address, wallet_address, username, role, status, email_verified, 
            phone_verified, two_factor_enabled, two_factor_secret, last_login, 
            login_attempts, locked_until, reset_token, reset_token_expires, 
            verification_token, verification_token_expires, created_at, updated_at
        FROM users WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    match row {
        Some(r) => Ok(Some(row_to_user(&r)?)),
        None => Ok(None),
    }
}

/// Get user by email
pub async fn get_user_by_email(db: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            id, email, password_hash, first_name, last_name, phone, date_of_birth, 
            nationality, address, wallet_address, username, role, status, email_verified, 
            phone_verified, two_factor_enabled, two_factor_secret, last_login, 
            login_attempts, locked_until, reset_token, reset_token_expires, 
            verification_token, verification_token_expires, created_at, updated_at
        FROM users WHERE LOWER(email) = LOWER($1)
        "#
    )
    .bind(email)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    match row {
        Some(r) => Ok(Some(row_to_user(&r)?)),
        None => Ok(None),
    }
}

/// Update user password
pub async fn update_user_password(db: &PgPool, user_id: Uuid, new_password_hash: &str) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE users 
        SET password_hash = $1, updated_at = $2
        WHERE id = $3
        "#
    )
    .bind(new_password_hash)
    .bind(Utc::now())
    .bind(user_id)
    .execute(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Update last login timestamp
pub async fn update_last_login(db: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE users 
        SET last_login = $1, updated_at = $2, login_attempts = 0
        WHERE id = $3
        "#
    )
    .bind(Utc::now())
    .bind(Utc::now())
    .bind(user_id)
    .execute(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Increment login attempts
pub async fn increment_login_attempts(db: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE users 
        SET login_attempts = login_attempts + 1, updated_at = $1
        WHERE id = $2
        "#
    )
    .bind(Utc::now())
    .bind(user_id)
    .execute(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Lock user account until specified time
pub async fn lock_user_account(db: &PgPool, user_id: Uuid, lock_until: DateTime<Utc>) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE users 
        SET locked_until = $1, updated_at = $2
        WHERE id = $3
        "#
    )
    .bind(lock_until)
    .bind(Utc::now())
    .bind(user_id)
    .execute(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}