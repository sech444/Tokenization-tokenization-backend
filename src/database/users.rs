
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