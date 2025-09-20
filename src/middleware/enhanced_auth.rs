// src/middleware/enhanced_auth.rs

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    database::users as db_users,
    models::user::{User, UserRole, UserStatus},
    utils::errors::{AppError, AppResult},
    AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedClaims {
    pub user_id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub permissions: Vec<Permission>,
    pub session_id: Uuid,
    pub iat: i64,
    pub exp: i64,
    pub aud: String,
    pub iss: String,
    pub device_id: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    // User permissions
    ReadProfile,
    UpdateProfile,
    DeleteProfile,

    // Project permissions
    CreateProject,
    ReadProject,
    UpdateProject,
    DeleteProject,
    ApproveProject,
    TokenizeProject,

    // Token permissions
    CreateToken,
    ReadToken,
    UpdateToken,
    DeleteToken,
    MintToken,
    BurnToken,

    // Trading permissions
    CreateOrder,
    ExecuteTrade,
    ViewTrades,
    CancelOrder,

    // Admin permissions
    ManageUsers,
    ViewAllProjects,
    ViewAllTokens,
    SystemAdmin,
    ComplianceReview,

    // KYC permissions
    SubmitKyc,
    ReviewKyc,
    ApproveKyc,
    RejectKyc,

    // Financial permissions
    ViewFinancials,
    ProcessPayments,
    ViewTransactions,

    // Marketplace permissions
    CreateListing,
    ViewListings,
    ManageListings,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: User,
    pub claims: EnhancedClaims,
    pub permissions: HashSet<Permission>,
    pub session_id: Uuid,
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: Option<String>,
}

pub struct RolePermissions;

impl RolePermissions {
    pub fn get_permissions(role: &UserRole) -> HashSet<Permission> {
        let mut permissions = HashSet::new();

        match role {
            UserRole::Admin => {
                // Admin has all permissions
                permissions.extend([
                    Permission::ReadProfile, Permission::UpdateProfile, Permission::DeleteProfile,
                    Permission::CreateProject, Permission::ReadProject, Permission::UpdateProject,
                    Permission::DeleteProject, Permission::ApproveProject, Permission::TokenizeProject,
                    Permission::CreateToken, Permission::ReadToken, Permission::UpdateToken,
                    Permission::DeleteToken, Permission::MintToken, Permission::BurnToken,
                    Permission::CreateOrder, Permission::ExecuteTrade, Permission::ViewTrades,
                    Permission::CancelOrder, Permission::ManageUsers, Permission::ViewAllProjects,
                    Permission::ViewAllTokens, Permission::SystemAdmin, Permission::ComplianceReview,
                    Permission::SubmitKyc, Permission::ReviewKyc, Permission::ApproveKyc,
                    Permission::RejectKyc, Permission::ViewFinancials, Permission::ProcessPayments,
                    Permission::ViewTransactions, Permission::CreateListing, Permission::ViewListings,
                    Permission::ManageListings,
                ]);
            }
            UserRole::ProjectManager => {
                permissions.extend([
                    Permission::ReadProfile, Permission::UpdateProfile,
                    Permission::CreateProject, Permission::ReadProject, Permission::UpdateProject,
                    Permission::TokenizeProject, Permission::CreateToken, Permission::ReadToken,
                    Permission::UpdateToken, Permission::MintToken, Permission::CreateOrder,
                    Permission::ExecuteTrade, Permission::ViewTrades, Permission::CancelOrder,
                    Permission::SubmitKyc, Permission::ViewFinancials, Permission::ViewTransactions,
                    Permission::CreateListing, Permission::ViewListings, Permission::ManageListings,
                ]);
            }
            UserRole::Investor => {
                permissions.extend([
                    Permission::ReadProfile, Permission::UpdateProfile,
                    Permission::ReadProject, Permission::ReadToken,
                    Permission::CreateOrder, Permission::ExecuteTrade, Permission::ViewTrades,
                    Permission::CancelOrder, Permission::SubmitKyc, Permission::ViewTransactions,
                    Permission::ViewListings,
                ]);
            }
            UserRole::ComplianceOfficer => {
                permissions.extend([
                    Permission::ReadProfile, Permission::UpdateProfile,
                    Permission::ReadProject, Permission::ReadToken, Permission::ViewAllProjects,
                    Permission::ViewAllTokens, Permission::ComplianceReview, Permission::ReviewKyc,
                    Permission::ApproveKyc, Permission::RejectKyc, Permission::ViewFinancials,
                    Permission::ViewTransactions, Permission::ViewListings,
                ]);
            }
            UserRole::Moderator => {
                permissions.extend([
                    Permission::ReadProfile, Permission::UpdateProfile,
                    Permission::ReadProject, Permission::ReadToken, Permission::ViewAllProjects,
                    Permission::ViewAllTokens, Permission::ReviewKyc, Permission::ViewListings,
                    Permission::ManageListings,
                ]);
            }
            UserRole::Developer => {
                permissions.extend([
                    Permission::ReadProfile, Permission::UpdateProfile,
                    Permission::ReadProject, Permission::ReadToken, Permission::CreateToken,
                    Permission::UpdateToken, Permission::MintToken, Permission::BurnToken,
                    Permission::ViewAllProjects, Permission::ViewAllTokens, Permission::SystemAdmin,
                ]);
            }
            UserRole::User => {
                permissions.extend([
                    Permission::ReadProfile, Permission::UpdateProfile,
                    Permission::CreateProject, Permission::ReadProject, Permission::UpdateProject,
                    Permission::ReadToken, Permission::CreateOrder, Permission::ViewTrades,
                    Permission::CancelOrder, Permission::SubmitKyc, Permission::ViewTransactions,
                    Permission::ViewListings,
                ]);
            }
        }

        permissions
    }
}

pub struct AuthValidator;

impl AuthValidator {
    pub fn validate_jwt_token(token: &str, secret: &str) -> AppResult<EnhancedClaims> {
        let validation = Validation::new(Algorithm::HS256);
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        let token_data = decode::<EnhancedClaims>(&token, &decoding_key, &validation)
            .map_err(|e| {
                warn!("JWT validation failed: {}", e);
                AppError::Unauthorized("Invalid or expired token".to_string())
            })?;

        // Additional validation
        let now = chrono::Utc::now().timestamp();
        if token_data.claims.exp < now {
            return Err(AppError::Unauthorized("Token has expired".to_string()));
        }

        if token_data.claims.iat > now + 300 {
            return Err(AppError::Unauthorized("Token issued in the future".to_string()));
        }

        Ok(token_data.claims)
    }

    pub async fn validate_user_status(db: &sqlx::PgPool, user_id: Uuid) -> AppResult<User> {
        let user = db_users::get_user_by_id(db, &user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

        match user.status {
            UserStatus::Active => Ok(user),
            UserStatus::Suspended => {
                warn!("Suspended user {} attempted access", user_id);
                Err(AppError::Forbidden("Account suspended".to_string()))
            }
            UserStatus::Inactive => {
                warn!("Inactive user {} attempted access", user_id);
                Err(AppError::Forbidden("Account inactive".to_string()))
            }
            UserStatus::PendingVerification => {
                Err(AppError::Forbidden("Account pending verification".to_string()))
            }
        }
    }

    pub fn validate_permissions(
        required_permissions: &[Permission],
        user_permissions: &HashSet<Permission>,
    ) -> bool {
        required_permissions.iter().all(|p| user_permissions.contains(p))
    }

    pub fn validate_resource_access(
        user: &User,
        resource_owner_id: Option<Uuid>,
        required_permissions: &[Permission],
    ) -> bool {
        // Admin always has access
        if matches!(user.role, UserRole::Admin) {
            return true;
        }

        // If resource has an owner, check if user owns it
        if let Some(owner_id) = resource_owner_id {
            if user.id == owner_id {
                return true;
            }
        }

        // Check if user has required permissions
        let user_permissions = RolePermissions::get_permissions(&user.role);
        Self::validate_permissions(required_permissions, &user_permissions)
    }
}

/// Enhanced authentication middleware
pub async fn enhanced_auth_middleware<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let headers = request.headers();
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized("Invalid authorization format".to_string()));
    }

    let token = &auth_header[7..];

    // Validate JWT token
    let claims = AuthValidator::validate_jwt_token(token, &state.config.jwt.secret)?;

    // Validate user exists and is active
    let user = AuthValidator::validate_user_status(&state.db, claims.user_id).await?;

    // Get user permissions
    let permissions = RolePermissions::get_permissions(&user.role);

    // Extract request metadata
    let ip_address = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Create auth context
    let auth_context = AuthContext {
        user: user.clone(),
        claims: claims.clone(),
        permissions: permissions.clone(),
        session_id: claims.session_id,
        request_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        ip_address: ip_address.clone(),
        user_agent,
    };

    // Security logging
    info!(
        "Authenticated user {} ({}) from {} with permissions: {:?}",
        user.email, user.role, ip_address, permissions
    );

    // Add to request extensions
    request.extensions_mut().insert(user);
    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(auth_context);
    request.extensions_mut().insert(permissions);

    Ok(next.run(request).await)
}

/// Permission validation middleware factory
pub fn require_permissions(required: Vec<Permission>) -> impl Clone + Fn() -> axum::middleware::FromFn<impl Clone + Send + Sync + 'static, fn(axum::Extension<HashSet<Permission>>, axum::extract::Request, axum::middleware::Next) -> impl std::future::Future<Output = Result<axum::response::Response, AppError>> + Send> {
    move || {
        axum::middleware::from_fn(move |permissions: Extension<HashSet<Permission>>, request: axum::extract::Request, next: axum::middleware::Next| async move {
            if !AuthValidator::validate_permissions(&required, &permissions) {
                warn!("Access denied: missing required permissions {:?}", required);
                return Err(AppError::Forbidden(
                    "Insufficient permissions for this operation".to_string(),
                ));
            }
            Ok(next.run(request).await)
        })
    }
}

/// Role validation middleware factory
pub fn require_role(required_roles: Vec<UserRole>) -> impl Clone + Send + Sync + 'static {
    axum::middleware::from_fn(move |user: Extension<User>, request: axum::extract::Request, next: axum::middleware::Next| async move {
        if !required_roles.contains(&user.role) {
            warn!(
                "Access denied: user role {:?} not in required roles {:?}",
                user.role, required_roles
            );
            return Err(AppError::Forbidden(
                "Insufficient role privileges for this operation".to_string(),
            ));
        }
        Ok(next.run(request).await)
    })
}

/// Resource ownership validation middleware
pub async fn validate_resource_ownership<B>(
    Extension(user): Extension<User>,
    Extension(auth_context): Extension<AuthContext>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // This middleware assumes the resource owner ID is available in request extensions
    // It will be set by specific resource middleware (e.g., project middleware)

    let resource_owner_id = request.extensions().get::<Uuid>().copied();

    if let Some(owner_id) = resource_owner_id {
        if user.id != owner_id && !matches!(user.role, UserRole::Admin) {
            error!(
                "User {} attempted to access resource owned by {}",
                user.id, owner_id
            );
            return Err(AppError::Forbidden(
                "You don't have permission to access this resource".to_string(),
            ));
        }
    }

    Ok(next.run(request).await)
}

/// KYC requirement validation
pub async fn require_kyc_verification<B>(
    Extension(user): Extension<User>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    if !user.kyc_verified {
        warn!("User {} attempted KYC-required operation without verification", user.id);
        return Err(AppError::Forbidden(
            "KYC verification required for this operation".to_string(),
        ));
    }
    Ok(next.run(request).await)
}

/// Two-factor authentication requirement
pub async fn require_2fa<B>(
    Extension(user): Extension<User>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    if !user.two_factor_enabled {
        warn!("User {} attempted 2FA-required operation without 2FA enabled", user.id);
        return Err(AppError::Forbidden(
            "Two-factor authentication required for this operation".to_string(),
        ));
    }
    Ok(next.run(request).await)
}

/// Audit trail middleware for sensitive operations
pub async fn audit_trail<B>(
    Extension(auth_context): Extension<AuthContext>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start_time = Utc::now();

    info!(
        "AUDIT: User {} ({}) {} {} from {} at {}",
        auth_context.user.email,
        auth_context.user.role,
        method,
        uri,
        auth_context.ip_address,
        start_time
    );

    let response = next.run(request).await;
    let duration = Utc::now().signed_duration_since(start_time);

    info!(
        "AUDIT: Operation completed in {}ms with status {:?}",
        duration.num_milliseconds(),
        response.status()
    );

    Ok(response)
}

/// Create middleware stack for different endpoint types
pub mod middleware_stacks {
    use super::*;

    pub fn public_endpoint() -> Vec<axum::middleware::FromFn<impl Clone + Send + Sync + 'static, fn(axum::extract::Request, axum::middleware::Next) -> impl std::future::Future<Output = Result<axum::response::Response, AppError>> + Send>> {
        vec![]
    }

    pub fn authenticated_endpoint() -> axum::middleware::FromFn<impl Clone + Send + Sync + 'static, fn(axum::extract::State<AppState>, axum::extract::Request, axum::middleware::Next) -> impl std::future::Future<Output = Result<axum::response::Response, AppError>> + Send> {
        axum::middleware::from_fn(enhanced_auth_middleware)
    }

    pub fn admin_endpoint() -> Vec<axum::middleware::FromFn<impl Clone + Send + Sync + 'static, fn(axum::Extension<User>, axum::extract::Request, axum::middleware::Next) -> impl std::future::Future<Output = Result<axum::response::Response, AppError>> + Send>> {
        vec![
            axum::middleware::from_fn(enhanced_auth_middleware),
            require_role(vec![UserRole::Admin]),
            axum::middleware::from_fn(audit_trail),
        ]
    }

    pub fn sensitive_operation() -> Vec<axum::middleware::FromFn<impl Clone + Send + Sync + 'static, fn(axum::Extension<User>, axum::extract::Request, axum::middleware::Next) -> impl std::future::Future<Output = Result<axum::response::Response, AppError>> + Send>> {
        vec![
            axum::middleware::from_fn(enhanced_auth_middleware),
            axum::middleware::from_fn(require_kyc_verification),
            axum::middleware::from_fn(require_2fa),
            axum::middleware::from_fn(audit_trail),
        ]
    }
}
