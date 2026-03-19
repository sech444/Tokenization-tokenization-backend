// src/handlers/password_reset.rs
// UPDATED VERSION WITH EMAIL SENDING

use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::users::get_user_by_email, 
    services::email::EmailService,
    utils::errors::AppError, 
    AppState
};

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

#[derive(Deserialize)]
pub struct ValidateTokenRequest {
    pub token: String,
}

#[derive(Serialize)]
pub struct ValidateTokenResponse {
    pub valid: bool,
    pub message: String,
}

fn generate_reset_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<(StatusCode, Json<ForgotPasswordResponse>), AppError> {
    let user = match get_user_by_email(&state.db, &payload.email).await? {
        Some(u) => u,
        None => {
            // Security: Return success even if user doesn't exist
            return Ok((
                StatusCode::OK,
                Json(ForgotPasswordResponse {
                    message: "If an account exists with that email, a password reset link has been sent.".to_string(),
                }),
            ));
        }
    };

    let reset_token = generate_reset_token();
    let reset_token_expires = Utc::now() + Duration::hours(1);

    // Update user with reset token
    sqlx::query(
        r#"
        UPDATE users 
        SET reset_token = $1, reset_token_expires = $2, updated_at = $3
        WHERE id = $4
        "#,
    )
    .bind(&reset_token)
    .bind(reset_token_expires)
    .bind(Utc::now())
    .bind(user.id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Generate reset link
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let reset_link = format!("{}/reset-password?token={}", frontend_url, reset_token);

    // Try to send email, but don't fail if email service is not configured
    match EmailService::new() {
        Ok(email_service) => {
            let user_name = if user.first_name.is_some() && user.last_name.is_some() {
                Some(format!("{} {}", 
                    user.first_name.as_ref().unwrap(), 
                    user.last_name.as_ref().unwrap()
                ))
            } else {
                None
            };

            if let Err(e) = email_service
                .send_password_reset_email(
                    &user.email,
                    user_name.as_deref(),
                    &reset_link,
                )
                .await
            {
                tracing::warn!("Failed to send password reset email: {}", e);
                tracing::info!("Password reset link (email failed): {}", reset_link);
            } else {
                tracing::info!("Password reset email sent successfully to: {}", payload.email);
            }
        }
        Err(e) => {
            // Email service not configured - just log the link
            tracing::warn!("Email service not configured: {}", e);
            tracing::info!("Password reset link for {}: {}", payload.email, reset_link);
        }
    }

    Ok((
        StatusCode::OK,
        Json(ForgotPasswordResponse {
            message: "If an account exists with that email, a password reset link has been sent.".to_string(),
        }),
    ))
}

pub async fn validate_reset_token(
    State(state): State<AppState>,
    Json(payload): Json<ValidateTokenRequest>,
) -> Result<(StatusCode, Json<ValidateTokenResponse>), AppError> {
    let result = sqlx::query_as::<_, (Uuid, chrono::DateTime<Utc>)>(
        r#"
        SELECT id, reset_token_expires
        FROM users 
        WHERE reset_token = $1
        "#,
    )
    .bind(&payload.token)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    match result {
        Some((_, expires)) => {
            if Utc::now() > expires {
                Ok((
                    StatusCode::BAD_REQUEST,
                    Json(ValidateTokenResponse {
                        valid: false,
                        message: "Reset token has expired".to_string(),
                    }),
                ))
            } else {
                Ok((
                    StatusCode::OK,
                    Json(ValidateTokenResponse {
                        valid: true,
                        message: "Token is valid".to_string(),
                    }),
                ))
            }
        }
        None => Ok((
            StatusCode::BAD_REQUEST,
            Json(ValidateTokenResponse {
                valid: false,
                message: "Invalid reset token".to_string(),
            }),
        )),
    }
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<(StatusCode, Json<ResetPasswordResponse>), AppError> {
    let result = sqlx::query_as::<_, (Uuid, chrono::DateTime<Utc>)>(
        r#"
        SELECT id, reset_token_expires
        FROM users 
        WHERE reset_token = $1
        "#,
    )
    .bind(&payload.token)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::AuthenticationFailed("Invalid reset token".to_string()))?;

    let (user_id, expires) = result;

    if Utc::now() > expires {
        return Err(AppError::AuthenticationFailed("Reset token has expired".to_string()));
    }

    if payload.new_password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".to_string()));
    }

    let has_uppercase = payload.new_password.chars().any(|c| c.is_uppercase());
    let has_lowercase = payload.new_password.chars().any(|c| c.is_lowercase());
    let has_digit = payload.new_password.chars().any(|c| c.is_numeric());

    if !has_uppercase || !has_lowercase || !has_digit {
        return Err(AppError::BadRequest(
            "Password must contain uppercase, lowercase, and number".to_string(),
        ));
    }

    use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.new_password.as_bytes(), &salt)
        .map_err(|e| AppError::InternalServerError(format!("Failed to hash password: {}", e)))?
        .to_string();

    sqlx::query(
        r#"
        UPDATE users 
        SET password_hash = $1, 
            reset_token = NULL, 
            reset_token_expires = NULL,
            updated_at = $2
        WHERE id = $3
        "#,
    )
    .bind(&password_hash)
    .bind(Utc::now())
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Password reset successfully for user_id: {}", user_id);

    Ok((
        StatusCode::OK,
        Json(ResetPasswordResponse {
            message: "Password has been reset successfully".to_string(),
        }),
    ))
}