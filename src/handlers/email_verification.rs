// src/handlers/email_verification.rs

use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    services::email::EmailService,
    utils::errors::AppError,
    AppState,
};

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[derive(Serialize)]
pub struct VerifyEmailResponse {
    pub message: String,
    pub verified: bool,
}

#[derive(Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Serialize)]
pub struct ResendVerificationResponse {
    pub message: String,
}

/// Generate a secure random verification token
fn generate_verification_token() -> String {
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

/// Send verification email to user
pub async fn send_verification_email(
    db: &sqlx::PgPool,
    user_id: Uuid,
    email: &str,
    first_name: Option<&str>,
    last_name: Option<&str>,
) -> Result<(), AppError> {
    // Generate verification token
    let verification_token = generate_verification_token();
    let verification_token_expires = Utc::now() + Duration::hours(24); // 24 hour expiry

    // Update user with verification token
    sqlx::query(
        r#"
        UPDATE users 
        SET verification_token = $1, 
            verification_token_expires = $2,
            updated_at = $3
        WHERE id = $4
        "#,
    )
    .bind(&verification_token)
    .bind(verification_token_expires)
    .bind(Utc::now())
    .bind(user_id)
    .execute(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Generate verification link
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let verification_link = format!("{}/verify-email?token={}", frontend_url, verification_token);

    // Try to send email
    match EmailService::new() {
        Ok(email_service) => {
            let user_name = if first_name.is_some() && last_name.is_some() {
                Some(format!("{} {}", first_name.unwrap(), last_name.unwrap()))
            } else {
                None
            };

            if let Err(e) = email_service
                .send_verification_email(email, user_name.as_deref(), &verification_link)
                .await
            {
                tracing::warn!("Failed to send verification email: {}", e);
                tracing::info!("Verification link (email failed): {}", verification_link);
            } else {
                tracing::info!("Verification email sent successfully to: {}", email);
            }
        }
        Err(e) => {
            tracing::warn!("Email service not configured: {}", e);
            tracing::info!("Verification link for {}: {}", email, verification_link);
        }
    }

    Ok(())
}

/// Verify email with token
pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<(StatusCode, Json<VerifyEmailResponse>), AppError> {
    // Find user by verification token
    let result = sqlx::query_as::<_, (Uuid, chrono::DateTime<Utc>, String)>(
        r#"
        SELECT id, verification_token_expires, email
        FROM users 
        WHERE verification_token = $1
        "#,
    )
    .bind(&payload.token)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let (user_id, expires, email) = match result {
        Some(data) => data,
        None => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(VerifyEmailResponse {
                    message: "Invalid verification token".to_string(),
                    verified: false,
                }),
            ));
        }
    };

    // Check if token has expired
    if Utc::now() > expires {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(VerifyEmailResponse {
                message: "Verification token has expired. Please request a new one.".to_string(),
                verified: false,
            }),
        ));
    }

    // Update user as verified and clear token
    sqlx::query(
        r#"
        UPDATE users 
        SET email_verified = true,
            verification_token = NULL,
            verification_token_expires = NULL,
            updated_at = $1
        WHERE id = $2
        "#,
    )
    .bind(Utc::now())
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Email verified successfully for user: {}", email);

    Ok((
        StatusCode::OK,
        Json(VerifyEmailResponse {
            message: "Email verified successfully! You can now log in.".to_string(),
            verified: true,
        }),
    ))
}

/// Resend verification email
pub async fn resend_verification_email(
    State(state): State<AppState>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<(StatusCode, Json<ResendVerificationResponse>), AppError> {
    // Find user by email
    let user = sqlx::query_as::<_, (Uuid, String, bool, Option<String>, Option<String>)>(
        r#"
        SELECT id, email, email_verified, first_name, last_name
        FROM users 
        WHERE LOWER(email) = LOWER($1)
        "#,
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let (user_id, email, email_verified, first_name, last_name) = match user {
        Some(data) => data,
        None => {
            // Security: Don't reveal if email exists
            return Ok((
                StatusCode::OK,
                Json(ResendVerificationResponse {
                    message: "If an account exists with that email, a verification link has been sent.".to_string(),
                }),
            ));
        }
    };

    // Check if already verified
    if email_verified {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ResendVerificationResponse {
                message: "This email is already verified.".to_string(),
            }),
        ));
    }

    // Send new verification email
    send_verification_email(
        &state.db,
        user_id,
        &email,
        first_name.as_deref(),
        last_name.as_deref(),
    )
    .await?;

    Ok((
        StatusCode::OK,
        Json(ResendVerificationResponse {
            message: "If an account exists with that email, a verification link has been sent.".to_string(),
        }),
    ))
}