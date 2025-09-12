// tokenization-backend/src/handlers/auth.rs

use axum::{
    extract::{State, Query},
    Extension, Json,
    http::StatusCode,
    response::IntoResponse,
};
use crate::models::kyc;
use crate::utils::auth::hash_password;
// use crate::utils::auth::verify_password;

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, warn };

use crate::{
    AppState,
    utils::errors::{AppError, AppResult},
    models::{User, UserRole, UserStatus},
    handlers::admin::{UserListQuery, UserSummary},
    database::{queries, users},
};

use crate::utils::auth::Claims;

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

// JWT Claims structure
// #[derive(Debug, Serialize, Deserialize)]
// pub struct Claims {
//     pub sub: String, // Subject (user ID)
//     pub exp: usize,  // Expiration time
//     pub iat: usize,  // Issued at
//     pub user_id: Uuid,
//     pub email: String,
//     pub role: UserRole,
// }

// ===============================
// Password and JWT Utilities
// ===============================


/// Verify a password against its hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash)
        .map_err(|e| AppError::HashVerificationFailed(format!("Failed to verify password: {}", e)))
}

/// Generate a JWT token for a user
pub fn generate_jwt_token(user: &User, jwt_secret: &str) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Valid timestamp")
        .timestamp();

    let claims = Claims {
        user_id: user.id,  
        exp: expiration as usize,
        iat: Utc::now().timestamp() as usize,
        email: user.email.clone(),
        username: user.username.clone().unwrap_or_default(), 
        role: user.role.clone().to_string(),
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

// ===============================
// Auth Handlers
// ===============================

/// Register a new user
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Registration attempt for email: {}", payload.email);
    
    // Validate input
    validate_email(&payload.email)
        .map_err(|e| AppError::validation(e))?;
    
    validate_password(&payload.password)
        .map_err(|e| AppError::validation(e))?;
    
    validate_name(&payload.first_name, "First name")
        .map_err(|e| AppError::validation(&e))?;
    
    validate_name(&payload.last_name, "Last name")
        .map_err(|e| AppError::validation(&e))?;

    validate_name(&payload.username, "username")
        .map_err(|e| AppError::validation(&e))?;

    // Check if email already exists
    let existing_user = users::get_user_by_email(&state.db, &payload.email).await?;
    if existing_user.is_some() {
        return Err(AppError::validation("Email already registered"));
    }

    // Hash password
    let password_hash = hash_password(&payload.password)?;

    // Create user struct
    let new_user = User {
        id: Uuid::new_v4(),
        email: payload.email.clone(),
        password_hash,
        first_name: Some(payload.first_name.clone()),
        last_name: Some(payload.last_name.clone()),
        phone: payload.phone,
        date_of_birth: payload.date_of_birth,
        nationality: payload.nationality,
        address: payload.address.map(|addr| serde_json::Value::String(addr)),
        wallet_address: None,
        username: Some(payload.username.clone()),
        role: UserRole::User, // Default role
        status: UserStatus::PendingVerification, // Default status
        email_verified:  Some(false),
        phone_verified:  Some(false),
        two_factor_enabled:  Some(false),
        two_factor_secret: None,
        last_login: None,
        login_attempts: Some(0),
        locked_until: None,
        reset_token: None,
        reset_token_expires: None,
        verification_token: Some(Uuid::new_v4().to_string()),
        verification_token_expires: Some(Utc::now() + Duration::hours(24)),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Create user in database
    let user = users::create_user(&state.db, &new_user).await?;

    // Generate JWT token
    let token = generate_jwt_token(&user, &state.config.jwt.secret)?;
    let expires_at = Utc::now() + Duration::hours(24);

    let response = AuthResponse {
        success: true,
        user_id: user.id,
        email: user.email,
        first_name: user.first_name.unwrap_or_else(|| "".to_string()),
        last_name: user.last_name.unwrap_or_else(|| "".to_string()),
        role: user.role,
        token,
        expires_at,
    };

    info!("User registration successful for: {}", response.email);
    Ok((StatusCode::CREATED, Json(response)))
}

/// User login
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Login attempt for email: {}", payload.email);
    
    // Validate input
    validate_email(&payload.email)
        .map_err(|e| AppError::validation(e))?;
    
    if payload.password.is_empty() {
        return Err(AppError::validation("Password is required"));
    }

    // Find user by email
    let user = users::get_user_by_email(&state.db, &payload.email).await?
        .ok_or_else(|| {
            warn!("Login attempt with non-existent email: {}", payload.email);
            AppError::validation("Invalid email or password")
        })?;

    // Check if account is locked
    if let Some(locked_until) = user.locked_until {
        if locked_until > Utc::now() {
            return Err(AppError::validation("Account is temporarily locked. Please try again later."));
        }
    }

    // Verify password
    if !verify_password(&payload.password, &user.password_hash)? {
        warn!("Failed login attempt for email: {} - wrong password", payload.email);
        
        // Increment login attempts (you would implement this in your database layer)
        // users::increment_login_attempts(&state.db, &user.id).await?;
        
        return Err(AppError::validation("Invalid email or password"));
    }

    // Check user status
    match user.status {
        UserStatus::Suspended => {
            return Err(AppError::validation("Account is suspended. Please contact support."));
        },
        UserStatus::Inactive => {
            return Err(AppError::validation("Account is deactivated. Please contact support."));
        },
        _ => {}
    }
    println!("User status: {:?}", user.status);
    // Reset login attempts on successful login
    // users::reset_login_attempts(&state.db, &user.id).await?;

    // Generate JWT token
    let token = generate_jwt_token(&user, &state.config.jwt.secret)?;
    let expires_at = Utc::now() + Duration::hours(24);

    // Update last login timestamp
    // users::update_last_login(&state.db, &user.id).await?;

    let response = AuthResponse {
        success: true,
        user_id: user.id,
        email: user.email,
        first_name: user.first_name.unwrap_or_else(|| "".to_string()),
        last_name: user.last_name.unwrap_or_else(|| "".to_string()),
        role: user.role,
        token,
        expires_at,
    };
    println!("Login successful for: {}", response.email);
    
    info!("Login successful for: {}", response.email);
    Ok(Json(response))

    
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

// /// Change password
// pub async fn change_password(
//     State(state): State<AppState>,
//     Extension(current_user): Extension<User>,
//     Json(payload): Json<ChangePasswordRequest>,
// ) -> AppResult<Json<serde_json::Value>> {
//     // Validate new password
//     validate_password(&payload.new_password)
//         .map_err(|e| AppError::validation(e))?;

//     // Verify current password
//     if !verify_password(&payload.current_password, &current_user.password_hash)? {
//         return Err(AppError::validation("Current password is incorrect"));
//     }

//     // Hash new password
//     let new_password_hash = hash_password(&payload.new_password)?;

//     // Update password in database
//     users::update_user_password(&state.db, current_user.id, &new_password_hash).await?;

//     info!("Password changed for user {}", current_user.id);

//     Ok(Json(serde_json::json!({
//         "success": true,
//         "message": "Password changed successfully"
//     })))
// }

// /// Logout user
// pub async fn logout(
//     Extension(current_user): Extension<User>,
// ) -> AppResult<Json<serde_json::Value>> {
//     info!("User {} logged out", current_user.id);
    
//     // Note: JWT tokens are stateless, so we can't truly invalidate them on the server
//     // without maintaining a blacklist. For now, we rely on client-side token removal.
    
//     Ok(Json(serde_json::json!({
//         "success": true,
//         "message": "Logged out successfully"
//     })))
// }

// // ===============================
// // Admin Handlers
// // ===============================

// /// List all users (admin only)
// pub async fn list_users(
//     State(state): State<AppState>,
//     Extension(current_user): Extension<User>,
//     Query(query_params): Query<UserListQuery>,
// ) -> AppResult<Json<UserListResponse>> {
//     // Check admin permissions
//     if !matches!(current_user.role, UserRole::Admin) {
//         warn!("Non-admin user {} attempted to access user list", current_user.id);
//         return Err(AppError::Forbidden("Admin access required".into()));
//     }

//     let page = query_params.page.unwrap_or(1).max(1);
//     let limit = query_params.limit.unwrap_or(20).min(100);
    
//     let (users, total_count) = queries::list_users(&state.db, &query_params, page, limit).await?;
//     let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;

//     info!("Admin {} retrieved {} users (page {}/{})", 
//           current_user.id, users.len(), page, total_pages);

//     Ok(Json(UserListResponse {
//         users,
//         total_count: total_count as u32,
//         page,
//         limit,
//         total_pages,
//     }))
// }

// /// Get pending KYC requests (admin only)
// pub async fn pending_kyc(
//     State(state): State<AppState>,
//     Extension(current_user): Extension<User>,
// ) -> AppResult<Json<serde_json::Value>> {
//     if !matches!(current_user.role, UserRole::Admin) {
//         return Err(AppError::Forbidden("Admin access required".into()));
//     }

//     // Get pending KYC requests from database
//     let pending_requests = queries::get_pending_kyc(&state.db).await?;
//     let count = pending_requests.len();

//     info!("Admin {} retrieved {} pending KYC requests", current_user.id, count);

//     Ok(Json(serde_json::json!({
//         "pending_kyc": pending_requests,
//         "count": count
//     })))
// }

// /// Approve or reject KYC request (admin only)
// pub async fn approve_kyc(
//     State(state): State<AppState>,
//     Extension(current_user): Extension<User>,
//     Json(payload): Json<ApproveKycRequest>,
// ) -> AppResult<Json<serde_json::Value>> {
//     if !matches!(current_user.role, UserRole::Admin) {
//         return Err(AppError::Forbidden("Admin access required".into()));
//     }

//     // Verify user exists
//     let user: User = users::get_user_by_id(&state.db, &payload.user_id).await?
//         .ok_or_else(|| AppError::validation("User not found"))?;

//     // Update KYC status in database
//     queries::update_kyc_status(&state.db, kyc::UpdateKycParams {
//         user_id: payload.user_id,
//         approved: payload.approved,
//         notes: payload.notes.clone(),
//         approved_by: current_user.id,
//     }).await?;

//     let action = if payload.approved { "approved" } else { "rejected" };
//     info!("KYC {} for user {} by admin {}", action, payload.user_id, current_user.id);

//     Ok(Json(serde_json::json!({
//         "success": true,
//         "message": format!("KYC {} for user {}", action, payload.user_id),
//         "user_id": payload.user_id,
//         "approved": payload.approved,
//         "notes": payload.notes,
//         "approved_by": current_user.id,
//         "approved_at": Utc::now().to_rfc3339()
//     })))
// }

// Update these handler functions in your handlers/auth.rs

use crate::middleware::auth::{AuthenticatedUser, RequireAdmin};

/// Change password
pub async fn change_password(
    State(state): State<AppState>,
    AuthenticatedUser(current_user): AuthenticatedUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Validate new password
    validate_password(&payload.new_password)
        .map_err(|e| AppError::validation(e))?;

    // Verify current password
    if !verify_password(&payload.current_password, &current_user.password_hash)? {
        return Err(AppError::validation("Current password is incorrect"));
    }

    // Hash new password
    let new_password_hash = hash_password(&payload.new_password)?;

    // Update password in database
    users::update_user_password(&state.db, current_user.id, &new_password_hash).await?;

    info!("Password changed for user {}", current_user.id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Password changed successfully"
    })))
}

/// Logout user
pub async fn logout(
    AuthenticatedUser(current_user): AuthenticatedUser,
) -> AppResult<Json<serde_json::Value>> {
    info!("User {} logged out", current_user.id);
    
    // Note: JWT tokens are stateless, so we can't truly invalidate them on the server
    // without maintaining a blacklist. For now, we rely on client-side token removal.
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

/// List all users (admin only)
pub async fn list_users(
    State(state): State<AppState>,
    RequireAdmin(current_user): RequireAdmin,
    Query(query_params): Query<UserListQuery>,
) -> AppResult<Json<UserListResponse>> {
    let page = query_params.page.unwrap_or(1).max(1);
    let limit = query_params.limit.unwrap_or(20).min(100);
    
    let (users, total_count) = queries::list_users(&state.db, &query_params, page, limit).await?;
    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;

    info!("Admin {} retrieved {} users (page {}/{})", 
          current_user.id, users.len(), page, total_pages);

    Ok(Json(UserListResponse {
        users,
        total_count: total_count as u32,
        page,
        limit,
        total_pages,
    }))
}

/// Get pending KYC requests (admin only)
pub async fn pending_kyc(
    State(state): State<AppState>,
    RequireAdmin(current_user): RequireAdmin,
) -> AppResult<Json<serde_json::Value>> {
    // Get pending KYC requests from database
    let pending_requests = queries::get_pending_kyc(&state.db).await?;
    let count = pending_requests.len();

    info!("Admin {} retrieved {} pending KYC requests", current_user.id, count);

    Ok(Json(serde_json::json!({
        "pending_kyc": pending_requests,
        "count": count
    })))
}

/// Approve or reject KYC request (admin only)
pub async fn approve_kyc(
    State(state): State<AppState>,
    RequireAdmin(current_user): RequireAdmin,
    Json(payload): Json<ApproveKycRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify user exists
    let user: User = users::get_user_by_id(&state.db, &payload.user_id).await?
        .ok_or_else(|| AppError::validation("User not found"))?;

    // Update KYC status in database
    queries::update_kyc_status(&state.db, kyc::UpdateKycParams {
        user_id: payload.user_id,
        approved: payload.approved,
        notes: payload.notes.clone(),
        approved_by: current_user.id,
    }).await?;

    let action = if payload.approved { "approved" } else { "rejected" };
    info!("KYC {} for user {} by admin {}", action, payload.user_id, current_user.id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("KYC {} for user {}", action, payload.user_id),
        "user_id": payload.user_id,
        "approved": payload.approved,
        "notes": payload.notes,
        "approved_by": current_user.id,
        "approved_at": Utc::now().to_rfc3339()
    })))
}