

// tokenization-backend/src/handlers/auth.rs

use axum::{
    extract::{State, Query, Path},
    Extension, Json,
    http::StatusCode,
    response::IntoResponse,
};
use crate::models::user::UpdateUserRequest;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, warn};
use crate::{
    database::{queries, users}, handlers::{admin::{UserListQuery, UserSummary}}, models::{user::{User, UserRole, UserStatus}, UserResponse}, utils::errors::{AppError, AppResult}, AppState
};

// use crate::{
//     models::user::{ UpdateUserRequest},
   
// };

use crate::utils::auth::{Claims, hash_password, verify_password}; 

// use crate::utils::auth::hash_password;
// use crate::utils::auth::verify_password;

// ===============================
// Request/Response Structures
// ===============================

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {    
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub nationality: Option<String>,
    pub address: Option<String>,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub user_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserSummary>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Deserialize)]
pub struct ApproveKycRequest {
    pub user_id: Uuid,
    pub approved: bool,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}


// tokenization-backend/src/handlers/user.rs
/// Generate a JWT token for a user
pub fn generate_jwt_token(user: &User, jwt_secret: &str) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Valid timestamp")
        .timestamp();

    let claims = Claims {
        user_id: user.id,                    // Changed from 'sub' to 'user_id'
        email: user.email.clone(),
        username: user.username.clone().expect("REASON"),     // Make sure this field exists
        role: user.role.to_string(),         // Convert UserRole to String
        exp: expiration as usize,
        iat: Utc::now().timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Failed to generate JWT: {}", e)))
}

/// Verify and decode a JWT token
pub fn verify_jwt_token(token: &str, jwt_secret: &str) -> Result<Claims, AppError> {
    let validation = Validation::new(Algorithm::HS256);
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::InvalidToken(format!("Invalid JWT token: {}", e)))
}

// ===============================
// Validation Helpers
// ===============================

fn validate_email(email: &str) -> Result<(), &'static str> {
    if email.is_empty() {
        return Err("Email is required");
    }
    
    if !email.contains('@') || !email.contains('.') {
        return Err("Invalid email format");
    }
    
    let email_parts: Vec<&str> = email.split('@').collect();
    if email_parts.len() != 2 || email_parts[0].is_empty() || email_parts[1].is_empty() {
        return Err("Invalid email format");
    }
    
    Ok(())
}

fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.is_empty() {
        return Err("Password is required");
    }
    
    if password.len() < 8 {
        return Err("Password must be at least 8 characters");
    }
    
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one number");
    }
    
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter");
    }
    
    Ok(())
}

fn validate_name(name: &str, field_name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err(format!("{} is required", field_name));
    }
    
    if name.len() > 50 {
        return Err(format!("{} must be less than 50 characters", field_name));
    }
    
    Ok(())
}



/// Verify JWT token
pub async fn verify_token(
    State(state): State<AppState>,
    Json(payload): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.token.is_empty() {
        return Err(AppError::InvalidToken("Token is required".to_string()));
    }

    // Verify JWT token
    let claims = verify_jwt_token(&payload.token, &state.config.jwt.secret)?;
    
    // Check if token is expired
    let now = Utc::now().timestamp() as usize;
    if claims.exp < now {
        return Err(AppError::InvalidToken("Token has expired".to_string()));
    }

    // Verify user still exists and is active
    let user = users::get_user_by_id(&state.db, &claims.user_id).await?
        .ok_or_else(|| AppError::InvalidToken("User not found".to_string()))?;

    if matches!(user.status, UserStatus::Suspended | UserStatus::Inactive) {
        return Err(AppError::InvalidToken("User account is deactivated".to_string()));
    }

    Ok(Json(serde_json::json!({
        "valid": true,
        "user_id": user.id,
        "email": user.email,
        "first_name": user.first_name,
        "last_name": user.last_name,
        "role": user.role,
        "status": user.status,
        "expires_at": claims.exp
    })))
}





/// Get user profile by ID
pub async fn get_user_profile(
    State(state): State<crate::AppState>,
    claims: Claims,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    // Check if user is accessing their own profile or is an admin
    if claims.user_id != user_id && !is_admin(&claims) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, first_name, last_name, phone, 
               date_of_birth, nationality, address, wallet_address, username, role, status,
               email_verified, phone_verified, two_factor_enabled, two_factor_secret,
               last_login, login_attempts, locked_until, reset_token, reset_token_expires,
               verification_token, verification_token_expires, created_at, updated_at
        FROM users 
        WHERE id = $1 AND status != 'inactive'
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    match user {
        Some(user) => Ok(Json(UserResponse::from(user))),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

/// Get current user's profile
pub async fn get_current_user_profile(
    State(state): State<crate::AppState>,
    claims: Claims,
) -> Result<Json<UserResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, first_name, last_name, phone, 
               date_of_birth, nationality, address, wallet_address, username, role, status,
               email_verified, phone_verified, two_factor_enabled, two_factor_secret,
               last_login, login_attempts, locked_until, reset_token, reset_token_expires,
               verification_token, verification_token_expires, created_at, updated_at
        FROM users 
        WHERE id = $1
        "#,
    )
    .bind(claims.user_id)
    .fetch_optional(&state.db)
    .await?;

    match user {
        Some(user) => Ok(Json(UserResponse::from(user))),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

/// Update user profile
pub async fn update_user_profile(
    State(state): State<crate::AppState>,
    claims: Claims,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Check if user is updating their own profile or is an admin
    if claims.user_id != user_id && !is_admin(&claims) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    // Update user profile
    let updated_user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET first_name = COALESCE($2, first_name),
            last_name = COALESCE($3, last_name),
            phone = COALESCE($4, phone),
            nationality = COALESCE($5, nationality),
            date_of_birth = COALESCE($6, date_of_birth),
            address = COALESCE($7, address),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, email, password_hash, first_name, last_name, phone, 
                  date_of_birth, nationality, address, wallet_address, username, role, status,
                  email_verified, phone_verified, two_factor_enabled, two_factor_secret,
                  last_login, login_attempts, locked_until, reset_token, reset_token_expires,
                  verification_token, verification_token_expires, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(payload.first_name)
    .bind(payload.last_name)
    .bind(payload.phone)
    .bind(payload.nationality)
    .bind(payload.date_of_birth)
    .bind(payload.address)
    .bind(payload.username)
    .fetch_optional(&state.db)
    .await?;

    match updated_user {
        Some(user) => Ok(Json(UserResponse::from(user))),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

/// Update current user's profile (convenience endpoint)
pub async fn update_user_profile_current(
    State(state): State<crate::AppState>,
    claims: Claims,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user_id = claims.user_id;
    // Reuse the existing update logic but with current user's ID
    update_user_profile(State(state), claims, Path(user_id), Json(payload)).await
}

/// Delete user profile (soft delete by setting status to inactive)
pub async fn delete_user_profile(
    State(state): State<crate::AppState>,
    claims: Claims,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check if user is deleting their own profile or is an admin
    if claims.user_id != user_id && !is_admin(&claims) {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    let result = sqlx::query(
        r#"
        UPDATE users 
        SET status = 'inactive', updated_at = NOW()
        WHERE id = $1 AND status != 'inactive'
        "#,
    )
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
fn is_admin(claims: &Claims) -> bool {
    claims.role == "admin"  // Compare string with string
}
