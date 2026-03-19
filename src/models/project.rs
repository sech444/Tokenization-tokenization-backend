// src/models/project.rs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

// ===== 1. DEFINE THE ENUMS ONCE, AT THE TOP =====

/// Enum for project status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "project_status", rename_all = "snake_case")]
pub enum ProjectStatus {
    Draft,
    Submitted,
    PendingApproval,
    Approved,
    Rejected,
    Active,
    Funded,
    Completed,
    Cancelled,
    Closed,
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Draft => "draft",
            Self::Submitted => "submitted",
            Self::PendingApproval => "pending_approval",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Active => "active",
            Self::Funded => "funded",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Closed => "closed",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for ProjectStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "submitted" => Ok(Self::Submitted),
            "pending_approval" => Ok(Self::PendingApproval),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "active" => Ok(Self::Active),
            "funded" => Ok(Self::Funded),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            "closed" => Ok(Self::Closed),
            _ => Err(format!("Unknown project status: {}", s)),
        }
    }
}

/// Enum for project type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "project_type", rename_all = "snake_case")]
pub enum ProjectType {
    Residential,
    Commercial,
    Industrial,
    MixedUse,
    Land,
    Hospitality,
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Residential => "residential",
            Self::Commercial => "commercial",
            Self::Industrial => "industrial",
            Self::MixedUse => "mixed_use",
            Self::Land => "land",
            Self::Hospitality => "hospitality",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for ProjectType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "residential" => Ok(Self::Residential),
            "commercial" => Ok(Self::Commercial),
            "industrial" => Ok(Self::Industrial),
            "mixed_use" | "mixed" => Ok(Self::MixedUse), // Allow "mixed" as an alias
            "land" => Ok(Self::Land),
            "hospitality" => Ok(Self::Hospitality),
            _ => Err(format!("Unknown project type: {}", s)),
        }
    }
}

// ===== 2. DEFINE THE STRUCTS THAT USE THE ENUMS =====

/// Project stored in DB
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub project_type: ProjectType,
    pub status: ProjectStatus,
    pub owner_id: Uuid,
    pub location: Option<String>,
    pub property_address: Option<String>,
    pub total_value: i64,
    pub minimum_investment: i64,
    pub maximum_investment: Option<i64>,
    pub funds_raised: i64,
    pub investor_count: i32,
    pub expected_return: Option<Decimal>,
    pub investment_period_months: i32,
    pub property_details: serde_json::Value,
    pub legal_documents: Option<Vec<String>>,
    pub images: Option<Vec<String>>,
    pub is_tokenized: bool,
    pub token_contract_address: Option<String>,
    pub compliance_verified: bool,
    pub kyc_required: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API Response DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub project_type: ProjectType,
    pub status: ProjectStatus,
    pub owner_id: Uuid,
    pub location: Option<String>,
    pub property_address: Option<String>,
    pub total_value: i64,
    pub minimum_investment: i64,
    pub maximum_investment: Option<i64>,
    pub funds_raised: i64,
    pub investor_count: i32,
    pub expected_return: Option<Decimal>,
    pub investment_period_months: i32,
    pub property_details: serde_json::Value,
    pub legal_documents: Option<Vec<String>>,
    pub images: Option<Vec<String>>,
    pub is_tokenized: bool,
    pub token_contract_address: Option<String>,
    pub compliance_verified: bool,
    pub kyc_required: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
            description: project.description,
            project_type: project.project_type,
            status: project.status,
            owner_id: project.owner_id,
            location: project.location,
            property_address: project.property_address,
            total_value: project.total_value,
            minimum_investment: project.minimum_investment,
            maximum_investment: project.maximum_investment,
            funds_raised: project.funds_raised,
            investor_count: project.investor_count,
            expected_return: project.expected_return,
            investment_period_months: project.investment_period_months,
            property_details: project.property_details,
            legal_documents: project.legal_documents,
            images: project.images,
            is_tokenized: project.is_tokenized,
            token_contract_address: project.token_contract_address,
            compliance_verified: project.compliance_verified,
            kyc_required: project.kyc_required,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

/// Request DTO for creating a project
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[validate(length(min = 3, max = 100))]
    pub name: String,
    #[validate(length(min = 10, max = 2000))]
    pub description: String,
    pub project_type: ProjectType,
    #[validate(range(min = 1))]
    pub total_value: i64,
    #[validate(range(min = 1))]
    pub minimum_investment: i64,
    pub maximum_investment: Option<i64>,
    pub expected_return: Option<Decimal>,
    pub investment_period_months: i32,
    pub location: Option<String>,
    pub property_address: Option<String>,
    pub property_details: serde_json::Value,
    pub legal_documents: Option<Vec<String>>,
    pub images: Option<Vec<String>>,
    pub kyc_required: bool,
}

/// Request DTO for updating a project
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub project_type: Option<ProjectType>,
    pub status: Option<ProjectStatus>,
    pub total_value: Option<i64>,
    pub minimum_investment: Option<i64>,
    pub maximum_investment: Option<i64>,
    pub expected_return: Option<Decimal>,
    pub investment_period_months: Option<i32>,
    pub location: Option<String>,
    pub property_address: Option<String>,
    pub property_details: Option<serde_json::Value>,
    pub legal_documents: Option<Vec<String>>,
    pub images: Option<Vec<String>>,
    pub is_tokenized: Option<bool>,
    pub token_contract_address: Option<String>,
    pub compliance_verified: Option<bool>,
    pub kyc_required: Option<bool>,
}

/// Summary DTO (for dashboards/analytics)
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub total_projects: i64,
    pub active_projects: i64,
    pub funded_projects: i64,
    pub completed_projects: i64,
}