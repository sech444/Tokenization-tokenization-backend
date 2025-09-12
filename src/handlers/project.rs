

// tokenization-backend/src/handlers/project.rs

use crate::utils::errors::AppError;
use crate::models::project::ProjectType;
use crate::utils::auth::Claims;
use crate::utils::errors::AppResult;
use crate::AppState;
use rust_decimal::Decimal;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;


use crate::{
    database::projects as db_projects,
    models::project::ProjectStatus,
    models::user::{User, UserRole},
    // utils::errors::AppResult,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: String,
    pub project_type: ProjectType, // ✅ use enum directly
    pub location: Option<String>,
    pub property_address: Option<String>,
    pub total_value: i64,                 // in cents
    pub minimum_investment: i64,          // in cents
    pub maximum_investment: Option<i64>,  // in cents
    pub expected_return: Option<Decimal>, // %
    pub investment_period_months: i32,
    pub property_details: Option<serde_json::Value>,
    pub legal_documents: Vec<String>,
    pub images: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectFilters {
    pub project_type: Option<ProjectType>, // ✅ also use enum here
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub location: Option<String>,
    pub is_tokenized: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

// Implement Display and FromStr for ProjectStatus
impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectStatus::Draft => write!(f, "draft"),
            ProjectStatus::PendingApproval => write!(f, "pending_approval"),
            ProjectStatus::Approved => write!(f, "approved"),
            ProjectStatus::Rejected => write!(f, "rejected"),
            ProjectStatus::Active => write!(f, "active"),
            ProjectStatus::Funded => write!(f, "funded"),
            ProjectStatus::Completed => write!(f, "completed"),
            ProjectStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectType::Residential => write!(f, "residential"),
            ProjectType::Commercial => write!(f, "commercial"),
            ProjectType::Industrial => write!(f, "industrial"),
            ProjectType::Mixed => write!(f, "mixed"),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct TokenizeProjectRequest {
    pub token_name: String,
    pub token_symbol: String,
    pub total_supply: u64,
    pub price_per_token: f64,
}

pub async fn tokenize_project(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<TokenizeProjectRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify project exists
    let project = db_projects::get_project_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    // Check if user owns the project or is admin
    if project.owner_id != current_user.id && !matches!(current_user.role, UserRole::Admin) {
        return Err(AppError::forbidden(
            "You don't have permission to tokenize this project".to_string(),
        ));
    }

    // ✅ Placeholder implementation
    Ok(Json(serde_json::json!({
        "message": format!("Project '{}' tokenization initiated", project.name),
        "project_id": id,
        "token_name": payload.token_name,
        "token_symbol": payload.token_symbol,
        "total_supply": payload.total_supply,
        "price_per_token": payload.price_per_token,
        "status": "success"
    })))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(project_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Step 1: Fetch the project
    let project = db_projects::get_project(&state.db, project_id)
        .await?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    // Step 2: Ownership / role check
    if project.owner_id != user.id && !matches!(user.role, UserRole::Admin) {
        return Err(AppError::forbidden("You can only delete your own projects"));
    }

    // Step 3: Status check
    if matches!(
        project.status,
        ProjectStatus::Active | ProjectStatus::Funded
    ) {
        return Err(AppError::bad_request(
            "Cannot delete active or funded projects",
        ));
    }

    // Step 4: Tokenization check
    if project.is_tokenized {
        return Err(AppError::bad_request("Cannot delete tokenized projects"));
    }

    // Step 5: Delete
    db_projects::delete_project_by_id(&state.db, project_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List all projects
pub async fn list_projects(
    State(state): State<AppState>,
    Query(filters): Query<ProjectFilters>,
) -> Result<impl IntoResponse, AppError> {
    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    let projects = db_projects::list_projects(&state.db, limit, offset).await?;
    let total = db_projects::count_projects(&state.db).await?;
    let total_pages = (total + limit - 1) / limit;

    Ok(Json(serde_json::json!({
        "data": projects,
        "page": page,
        "limit": limit,
        "total": total,
        "total_pages": total_pages
    })))
}

/// Create new project
pub async fn create_project(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    // ✅ validation without .as_str()
    if payload.name.is_empty() || payload.description.is_empty() {
        return Err(AppError::BadRequest(
            "Project name and description are required".into(),
        ));
    }

    if payload.total_value <= 0 {
        return Err(AppError::BadRequest("Total value must be positive".into()));
    }

    if payload.minimum_investment <= 0 {
        return Err(AppError::BadRequest(
            "Minimum investment must be positive".into(),
        ));
    }

    if let Some(max_inv) = payload.maximum_investment {
        if max_inv < payload.minimum_investment {
            return Err(AppError::BadRequest(
                "Maximum investment cannot be less than minimum investment".into(),
            ));
        }
    }

    if payload.investment_period_months <= 0 {
        return Err(AppError::BadRequest(
            "Investment period must be positive".into(),
        ));
    }

    let project = db_projects::create_project(&state.db, &payload, claims.user_id).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

/// Get project by ID
pub async fn get_project(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let project = db_projects::get_project(&state.db, project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".into()))?;

    Ok(Json(project))
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

// pub async fn update_project_status(
//     State(state): State<AppState>,
//     Extension(current_user): Extension<User>,
//     Path(project_id): Path<Uuid>,
//     Json(payload): Json<UpdateProjectStatusRequest>,
// ) -> AppResult<Json<String>> {
//     ensure_admin(&current_user)?;

//     queries::update_project_status(&state.db, project_id, payload.status).await?;

//     Ok(Json("Project status updated".to_string()))
    
// }

