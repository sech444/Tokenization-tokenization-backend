// src/handlers/admin.rs

use axum::{
    extract::{Path, Query, State},
    response::Json,
    Extension,
};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::database::{projects, queries, transactions};
use crate::{
    AppState,
    // database::queries: as db,
    models::{
        kyc::{RiskLevel, VerificationStatus},
        project::ProjectStatus, 
        user::{User, UserRole, UserStatus},
    },
    utils::errors::{AppError, AppResult},
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

// #[derive(Debug, Serialize)]
// pub struct UserSummary {
//     pub id: Uuid,
//     pub email: String,
//     pub role: String,
//     pub status: UserStatus,
//     pub created_at: DateTime<Utc>,
//     pub last_login: Option<DateTime<Utc>>,
//     // pub role: String,
//     pub is_active: Option<bool>,
//     // pub created_at: chrono::NaiveDateTime,
// }

// #[derive(Debug, Serialize)]
#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub email: String,
    pub role: Option<String>,    // Change to Option<String>
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
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

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
}

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

