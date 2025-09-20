// src/middleware/project_validation.rs

use axum::{
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    database::projects as db_projects,
    models::{
        project::{Project, ProjectStatus, ProjectType},
        user::{User, UserRole},
    },
    utils::errors::{AppError, AppResult},
    AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectValidationConfig {
    pub min_project_value: i64,
    pub max_project_value: i64,
    pub required_documents: Vec<String>,
    pub allowed_project_types: Vec<ProjectType>,
    pub require_kyc_approval: bool,
    pub min_investment_period_months: i32,
    pub max_investment_period_months: i32,
}

impl Default for ProjectValidationConfig {
    fn default() -> Self {
        Self {
            min_project_value: 10_000_00,      // $10,000 minimum
            max_project_value: 100_000_000_00, // $100M maximum
            required_documents: vec![
                "property_deed".to_string(),
                "valuation_report".to_string(),
                "legal_opinion".to_string(),
            ],
            allowed_project_types: vec![
                ProjectType::Residential,
                ProjectType::Commercial,
                ProjectType::Industrial,
                ProjectType::Mixed,
            ],
            require_kyc_approval: true,
            min_investment_period_months: 6,
            max_investment_period_months: 120, // 10 years max
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub project: Project,
    pub validation_score: u8, // 0-100
}

pub struct ProjectValidator {
    config: ProjectValidationConfig,
}

impl ProjectValidator {
    pub fn new(config: Option<ProjectValidationConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    /// Validate project for tokenization
    pub async fn validate_for_tokenization(
        &self,
        project: &Project,
        user: &User,
    ) -> ProjectValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut score = 100u8;

        // 1. Status validation - CRITICAL
        match project.status {
            ProjectStatus::Approved => {
                info!("Project {} has approved status", project.id);
            }
            ProjectStatus::Active => {
                info!("Project {} is active and can be tokenized", project.id);
            }
            ProjectStatus::Draft | ProjectStatus::PendingApproval => {
                errors.push("Project must be approved before tokenization".to_string());
                score = score.saturating_sub(50);
            }
            ProjectStatus::Rejected | ProjectStatus::Cancelled => {
                errors.push("Cannot tokenize rejected or cancelled projects".to_string());
                score = 0;
            }
            ProjectStatus::Funded | ProjectStatus::Completed => {
                errors.push("Project is already funded or completed".to_string());
                score = score.saturating_sub(30);
            }
        }

        // 2. Ownership validation - CRITICAL
        if project.owner_id != user.id && !matches!(user.role, UserRole::Admin) {
            errors.push("User does not own this project".to_string());
            score = 0;
        }

        // 3. Project value validation
        if project.total_value < self.config.min_project_value {
            errors.push(format!(
                "Project value ${} is below minimum ${}",
                project.total_value / 100,
                self.config.min_project_value / 100
            ));
            score = score.saturating_sub(20);
        }

        if project.total_value > self.config.max_project_value {
            warnings.push(format!(
                "Project value ${} is above typical maximum ${}",
                project.total_value / 100,
                self.config.max_project_value / 100
            ));
            score = score.saturating_sub(5);
        }

        // 4. Project type validation
        if !self
            .config
            .allowed_project_types
            .contains(&project.project_type)
        {
            errors.push(format!(
                "Project type {:?} is not allowed for tokenization",
                project.project_type
            ));
            score = score.saturating_sub(30);
        }

        // 5. Investment validation
        if project.minimum_investment <= 0 {
            errors.push("Minimum investment must be positive".to_string());
            score = score.saturating_sub(15);
        }

        if project.minimum_investment > project.total_value / 10 {
            warnings.push("Minimum investment is more than 10% of total value".to_string());
            score = score.saturating_sub(5);
        }

        // 6. Investment period validation
        if project.investment_period_months < self.config.min_investment_period_months {
            errors.push(format!(
                "Investment period {} months is below minimum {} months",
                project.investment_period_months, self.config.min_investment_period_months
            ));
            score = score.saturating_sub(15);
        }

        if project.investment_period_months > self.config.max_investment_period_months {
            errors.push(format!(
                "Investment period {} months exceeds maximum {} months",
                project.investment_period_months, self.config.max_investment_period_months
            ));
            score = score.saturating_sub(10);
        }

        // 7. Already tokenized check
        if project.is_tokenized {
            errors.push("Project is already tokenized".to_string());
            score = score.saturating_sub(40);
        }

        // 8. Required fields validation
        if project.property_address.is_none()
            || project.property_address.as_ref().unwrap().is_empty()
        {
            errors.push("Property address is required".to_string());
            score = score.saturating_sub(10);
        }

        if project.location.is_none() || project.location.as_ref().unwrap().is_empty() {
            warnings.push("Project location should be specified".to_string());
            score = score.saturating_sub(5);
        }

        // 9. KYC validation if required
        if self.config.require_kyc_approval {
            if !user.kyc_verified {
                errors.push(
                    "User must complete KYC verification before tokenizing projects".to_string(),
                );
                score = score.saturating_sub(50);
            }
        }

        // 10. Expected return validation
        if let Some(expected_return) = project.expected_return {
            if expected_return < rust_decimal::Decimal::new(1, 0) {
                // Less than 1%
                warnings.push("Expected return seems unusually low".to_string());
                score = score.saturating_sub(5);
            }
            if expected_return > rust_decimal::Decimal::new(50, 0) {
                // More than 50%
                warnings.push(
                    "Expected return seems unusually high - may require additional scrutiny"
                        .to_string(),
                );
                score = score.saturating_sub(5);
            }
        }

        ProjectValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            project: project.clone(),
            validation_score: score,
        }
    }

    /// Validate project creation/update
    pub fn validate_project_data(&self, project: &Project) -> ProjectValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut score = 100u8;

        // Basic field validation
        if project.name.is_empty() {
            errors.push("Project name is required".to_string());
            score = score.saturating_sub(20);
        } else if project.name.len() < 3 {
            errors.push("Project name must be at least 3 characters".to_string());
            score = score.saturating_sub(10);
        } else if project.name.len() > 100 {
            errors.push("Project name must be less than 100 characters".to_string());
            score = score.saturating_sub(5);
        }

        if project.description.is_empty() {
            errors.push("Project description is required".to_string());
            score = score.saturating_sub(15);
        } else if project.description.len() < 50 {
            warnings.push(
                "Project description should be more detailed (recommended 50+ characters)"
                    .to_string(),
            );
            score = score.saturating_sub(5);
        }

        // Value validation
        if project.total_value <= 0 {
            errors.push("Total project value must be positive".to_string());
            score = 0;
        }

        if project.minimum_investment <= 0 {
            errors.push("Minimum investment must be positive".to_string());
            score = score.saturating_sub(20);
        }

        if let Some(max_investment) = project.maximum_investment {
            if max_investment < project.minimum_investment {
                errors
                    .push("Maximum investment cannot be less than minimum investment".to_string());
                score = score.saturating_sub(15);
            }
        }

        ProjectValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            project: project.clone(),
            validation_score: score,
        }
    }
}

/// Middleware to validate project exists and user has access
pub async fn validate_project_access<B>(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(path_params): Path<HashMap<String, String>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // Extract project_id from path
    let project_id = path_params
        .get("id")
        .or_else(|| path_params.get("project_id"))
        .ok_or_else(|| AppError::BadRequest("Project ID not found in path".to_string()))?;

    let project_uuid = project_id
        .parse::<Uuid>()
        .map_err(|_| AppError::BadRequest("Invalid project ID format".to_string()))?;

    // Get project from database
    let project = db_projects::get_project(&state.db, project_uuid)
        .await
        .map_err(|e| {
            error!("Failed to fetch project {}: {}", project_uuid, e);
            AppError::InternalServerError("Failed to fetch project".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    // Check access permissions
    let has_access = match user.role {
        UserRole::Admin => true,
        UserRole::ProjectManager | UserRole::User => project.owner_id == user.id,
        _ => false,
    };

    if !has_access {
        warn!(
            "User {} attempted to access project {} without permission",
            user.id, project_uuid
        );
        return Err(AppError::Forbidden(
            "You don't have permission to access this project".to_string(),
        ));
    }

    // Add project to request extensions for downstream handlers
    let mut request = request;
    request.extensions_mut().insert(project);

    Ok(next.run(request).await)
}

/// Middleware to validate project can be tokenized
pub async fn validate_tokenization_eligibility<B>(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Extension(project): Extension<Project>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let validator = ProjectValidator::new(None);
    let validation_result = validator.validate_for_tokenization(&project, &user).await;

    if !validation_result.is_valid {
        error!(
            "Project {} failed tokenization validation: {:?}",
            project.id, validation_result.errors
        );
        return Err(AppError::BadRequest(format!(
            "Project is not eligible for tokenization: {}",
            validation_result.errors.join(", ")
        )));
    }

    if !validation_result.warnings.is_empty() {
        warn!(
            "Project {} tokenization warnings: {:?}",
            project.id, validation_result.warnings
        );
    }

    info!(
        "Project {} passed tokenization validation with score {}",
        project.id, validation_result.validation_score
    );

    // Add validation result to request extensions
    let mut request = request;
    request.extensions_mut().insert(validation_result);

    Ok(next.run(request).await)
}

/// Check if user has any approved projects
pub async fn user_has_approved_projects(db: &sqlx::PgPool, user_id: Uuid) -> AppResult<bool> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count
        FROM projects
        WHERE owner_id = $1
        AND status IN ('approved', 'active')
        AND NOT is_tokenized
        "#,
        user_id
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        error!("Failed to check user projects: {}", e);
        AppError::InternalServerError("Failed to check user projects".to_string())
    })?;

    Ok(count.unwrap_or(0) > 0)
}

/// Middleware to ensure user has approved projects before token operations
pub async fn require_approved_projects<B>(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // Skip check for admins
    if matches!(user.role, UserRole::Admin) {
        return Ok(next.run(request).await);
    }

    let has_projects = user_has_approved_projects(&state.db, user.id).await?;

    if !has_projects {
        warn!(
            "User {} attempted token operation without approved projects",
            user.id
        );
        return Err(AppError::BadRequest(
            "You must have at least one approved project before creating tokens".to_string(),
        ));
    }

    Ok(next.run(request).await)
}
