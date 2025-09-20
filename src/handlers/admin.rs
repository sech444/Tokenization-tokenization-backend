// src/handlers/admin.rs

use axum::{
    extract::{Path, Query, State, Extension},
    response::Json,
};

// use axum::{
//     extract::{State, Query},
//     Json,
// };
use crate::{
    AppState,
    middleware::auth::RequireAdmin,
    utils::errors::{AppError, AppResult},
};
// use serde::Deserialize;
// use chrono::Utc;

use crate::handlers::auth::ApproveKycRequest;
// use sqlx::PgPool;
// use crate::middleware::auth::RequireAdmin;
use crate::models::kyc::UpdateKycParams;
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::database::{projects, queries, transactions};
use tracing::info;
 use crate::database::users;
// use tracing::warn;
use crate::{
    // AppState,
    // database::queries: as db,
    models::{
        kyc::{RiskLevel, VerificationStatus},
        project::ProjectStatus, 
        user::{User, UserRole, UserStatus},
    },
    
};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------



#[derive(Debug, Serialize)]
pub struct AdminDashboardResponse {
    pub totals: TotalsBlock,
    pub financials: FinancialsBlock,
}

#[derive(Debug, Serialize)]
pub struct TotalsBlock {
    pub total_users: i64,
    pub active_users: i64,
    pub total_projects: i64,
    pub active_projects: i64,
    pub total_tokens: i64,
    pub total_transactions: i64,
}

#[derive(Debug, Serialize)]
pub struct FinancialsBlock {
    pub completed_volume: Option<f64>,
}





// src/handlers/admin.rs - Add the missing derives

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserSummary {
    pub id: Uuid,
    pub email: String,
    pub status: String,   // changed from UserStatus
    pub last_login: Option<DateTime<Utc>>,
    pub role: String,     // changed from UserRole
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserSummary>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
    pub email: Option<String>,
    pub is_active: Option<bool>,
}

// #[derive(Debug, Deserialize)]
// pub struct UpdateUserRequest {
//     pub role: Option<UserRole>,
//     pub status: Option<UserStatus>,
// }

#[derive(Debug, Serialize)]
pub struct UpdateUserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub updated_at: DateTime<Utc>,
}



#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct KycVerificationSummary {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub verification_status: String,
    pub risk_level: String,
    pub verification_score: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct KycListResponse {
    pub verifications: Vec<KycVerificationSummary>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}




#[derive(Debug, Deserialize)]
pub struct KycListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<VerificationStatus>,
    pub risk_level: Option<RiskLevel>,
    pub requires_review: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectStatusRequest {
    pub status: ProjectStatus,
}

// ---------------------------------------------------------------------------
// Admin-only guard
// ---------------------------------------------------------------------------

fn ensure_admin(current_user: &User) -> AppResult<()> {
    if matches!(
        current_user.role,
        UserRole::Admin | UserRole::ComplianceOfficer
    ) {
        Ok(())
    } else {
        Err(AppError::Forbidden("Admin access required".to_string()))
    }
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

pub async fn admin_dashboard(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> AppResult<Json<AdminDashboardResponse>> {
    ensure_admin(&current_user)?;

    let total_users = queries::count_users(&state.db).await?;
    let active_users = queries::count_active_users(&state.db).await?;
    let total_projects = projects::count_projects(&state.db).await?;
    let active_projects = queries::count_active_projects(&state.db).await?;
    let total_tokens = transactions::count_tokens(&state.db).await?;
    let total_transactions = transactions::count_transactions(&state.db).await?;
    let completed_volume = transactions::completed_volume(&state.db).await?;

   
    Ok(Json(AdminDashboardResponse {
        totals: TotalsBlock {
            total_users,
            active_users,
            total_projects,
            active_projects,
            total_tokens,
            total_transactions,
        },
        financials: FinancialsBlock { completed_volume: Some(completed_volume) },
    }))
    }



pub async fn list_kyc_verifications(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(q): Query<KycListQuery>,
) -> AppResult<Json<KycListResponse>> {
    ensure_admin(&current_user)?;

    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(20).min(100);

    let (verifications, total_count) =
        queries::list_kyc_verifications(&state.db, &q, page, limit).await?;

    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as u32;

    Ok(Json(KycListResponse {
        verifications,
        total_count: total_count as u32,
        page,
        limit,
        total_pages,
    }))
}


    #[derive(Deserialize)]
    pub struct Pagination {
        pub page: Option<u32>,
        pub limit: Option<u32>,
    }



    /// Get pending KYC requests (admin only)
    pub async fn pending_kyc(
        State(state): State<AppState>,
        RequireAdmin(admin): RequireAdmin,
    ) -> AppResult<Json<serde_json::Value>> {
        let pending_requests = queries::get_pending_kyc(&state.db).await?;
        let count = pending_requests.len();

        info!("Admin {} retrieved {} pending KYC requests", admin.id, count);

        Ok(Json(serde_json::json!({
            "pending_kyc": pending_requests,
            "count": count
        })))
    }

    /// Approve or reject KYC request (admin only)
    pub async fn approve_kyc(
        State(state): State<AppState>,
        RequireAdmin(admin): RequireAdmin,
        Json(payload): Json<ApproveKycRequest>,
    ) -> AppResult<Json<serde_json::Value>> {
        let user = users::get_user_by_id(&state.db, &payload.user_id).await?
            .ok_or_else(|| AppError::validation("User not found"))?;

        queries::update_kyc_status(&state.db, UpdateKycParams {
            user_id: payload.user_id,
            approved: payload.approved,
            notes: payload.notes.clone(),
            approved_by: admin.id,
        }).await?;

        let action = if payload.approved { "approved" } else { "rejected" };

        Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("KYC {} for user {}", action, payload.user_id),
            "user_id": payload.user_id,
            "approved": payload.approved,
            "notes": payload.notes,
            "approved_by": admin.id,
            "approved_at": Utc::now().to_rfc3339()
        })))
    }



