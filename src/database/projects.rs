

// src/database/projects.rs

use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::utils::errors::AppResult;

use crate::{
    models::project::{Project, ProjectResponse, ProjectStatus, ProjectType},
    utils::errors::AppError,
};

/// List projects with pagination (simple)
pub async fn list_projects(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<ProjectResponse>, AppError> {
    let projects = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Convert Project to ProjectResponse
    let responses = projects.into_iter().map(|p| p.into()).collect();
    Ok(responses)
}

/// Count all projects
pub async fn count_projects(pool: &PgPool) -> Result<i64, AppError> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM projects")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(row.get::<i64, _>("count"))
}

/// Insert a new project
pub async fn create_project(
    pool: &PgPool,
    payload: &crate::handlers::project::CreateProjectRequest,
    owner_id: Uuid,
) -> Result<ProjectResponse, AppError> {
    let project = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (
            name, description, project_type, owner_id, location, property_address,
            total_value, minimum_investment, maximum_investment, expected_return,
            investment_period_months, property_details, legal_documents, images
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.project_type)
    .bind(owner_id)
    .bind(&payload.location)
    .bind(&payload.property_address)
    .bind(payload.total_value)
    .bind(payload.minimum_investment)
    .bind(payload.maximum_investment)
    .bind(payload.expected_return)
    .bind(payload.investment_period_months)
    .bind(&payload.property_details)
    .bind(&payload.legal_documents)
    .bind(&payload.images)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(project.into())
}

/// Get a project by ID
pub async fn get_project(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Option<ProjectResponse>, AppError> {
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(project.map(|p| p.into()))
}

/// Delete a project
pub async fn delete_project_by_id(db: &PgPool, project_id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(project_id)
        .execute(db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

/// Update project tokenization
pub async fn update_project_tokenization(
    db: &PgPool,
    project_id: Uuid,
    contract_address: &str,
) -> Result<Project, AppError> {
    sqlx::query_as::<_, Project>(
        r#"
        UPDATE projects
        SET is_tokenized = true,
            token_contract_address = $1,
            updated_at = NOW()
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(contract_address)
    .bind(project_id)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))
}

/// Example of list with filters (dynamic conditions)
pub async fn list_projects_filtered(
    db: &PgPool,
    status: Option<ProjectStatus>,
    project_type: Option<ProjectType>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Project>, AppError> {
    sqlx::query_as::<_, Project>(
        r#"
        SELECT * FROM projects
        WHERE ($1::text IS NULL OR status::text = $1)
          AND ($2::text IS NULL OR project_type::text = $2)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(status.map(|s| s.to_string()))
    .bind(project_type.map(|pt| pt.to_string()))
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub async fn get_project_by_id(pool: &PgPool, id: &Uuid) -> AppResult<Option<Project>> {
    let query = r#"
        SELECT 
            id, name, description, project_type, status, owner_id, 
            location, property_address, total_value, minimum_investment, 
            maximum_investment, funds_raised, investor_count, expected_return, 
            investment_period_months, property_details, legal_documents, 
            images, is_tokenized, token_contract_address, compliance_verified, 
            kyc_required, created_at, updated_at
        FROM projects 
        WHERE id = $1
    "#;

    let row = sqlx::query(query)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(row
        .map(|r| -> Result<Project, AppError> {
            let project_type = r
                .get::<String, _>("project_type")
                .parse()
                .map_err(|_| AppError::ValidationError("Invalid project type".to_string()))?;

            let status = r
                .get::<String, _>("status")
                .parse()
                .map_err(|_| AppError::ValidationError("Invalid project status".to_string()))?;

            Ok(Project {
                id: r.get("id"),
                name: r.get("name"),
                description: r.get("description"),
                project_type,
                status,
                owner_id: r.get("owner_id"),
                location: r.get("location"),
                property_address: r.get("property_address"),
                total_value: r.get("total_value"),
                minimum_investment: r.get("minimum_investment"),
                maximum_investment: r.get("maximum_investment"),
                funds_raised: r.get("funds_raised"),
                investor_count: r.get("investor_count"),
                expected_return: r
                    .get::<Option<Decimal>, _>("expected_return")
                    .map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
                investment_period_months: r.get("investment_period_months"),
                property_details: r
                    .get::<Option<serde_json::Value>, _>("property_details")
                    .unwrap_or_else(|| serde_json::json!({})),
                legal_documents: r.get("legal_documents"),
                images: r.get("images"),
                is_tokenized: r.get("is_tokenized"),
                token_contract_address: r.get("token_contract_address"),
                compliance_verified: r.get("compliance_verified"),
                kyc_required: r.get("kyc_required"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
        })
        .transpose()?)
}
