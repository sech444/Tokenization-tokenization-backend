// tokenization-backend/src/handlers/kyc.rs
// Refactored: All DB access moved into crate::database::queries

use axum::{
    extract::{Multipart, Path, State},
    response::Json,
    Extension,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::{ queries };
use crate::{
    models::{
        kyc::{
            DocumentType, DocumentVerificationStatus, InitiateKycRequest,
            RiskLevel, VerificationStatus,
        },
        user::{User, UserRole},
    },
    utils::errors::{AppError, AppResult},
};

// use crate::models::kyc::{
//     DocumentType, DocumentVerificationStatus, InitiateKycRequest,
//     KycVerification, RiskLevel, VerificationStatus,
// };

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct KycStatusResponse {
    pub kyc_id: Uuid,
    pub user_id: Uuid,
    pub verification_status: VerificationStatus,
    pub risk_level: RiskLevel,
    pub documents_verified: bool,
    pub identity_verified: bool,
    pub address_verified: bool,
    pub phone_verified: bool,
    pub email_verified: bool,
    pub verification_score: Option<f32>,
    pub verification_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub required_documents: Vec<DocumentRequirement>,
    pub uploaded_documents: Vec<DocumentSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentRequirement {
    pub document_type: DocumentType,
    pub required: bool,
    pub description: String,
    pub uploaded: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub id: Uuid,
    pub document_type: DocumentType,
    pub verification_status: DocumentVerificationStatus,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentUploadResponse {
    pub document_id: Uuid,
    pub document_type: DocumentType,
    pub upload_status: String,
    pub verification_status: DocumentVerificationStatus,
    pub confidence_score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KycReviewRequest {
    pub approved: bool,
    pub risk_level: Option<RiskLevel>,
    pub notes: Option<String>,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KycListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<VerificationStatus>,
    pub risk_level: Option<RiskLevel>,
    pub requires_review: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KycListResponse {
    pub verifications: Vec<KycVerificationSummary>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct KycVerificationSummary {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub verification_status: VerificationStatus,
    pub risk_level: RiskLevel,
    pub verification_score: Option<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------





pub async fn initiate_kyc(
    State(state): State<crate::AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<InitiateKycRequest>,
) -> AppResult<Json<KycStatusResponse>> {
    if request.user_id != user.id
        && !matches!(user.role, UserRole::Admin | UserRole::ComplianceOfficer)
    {
        return Err(AppError::Forbidden(
            "You can only initiate KYC for yourself".to_string(),
        ));
    }

    let created = queries::initiate_kyc(&state.db, request.user_id).await?;
    get_kyc_status_internal(&state, created.user_id).await
}

pub async fn get_kyc_status(
    State(state): State<crate::AppState>,
    Extension(user): Extension<User>,
    Path(user_id): Path<Option<Uuid>>,
) -> AppResult<Json<KycStatusResponse>> {
    let target = user_id.unwrap_or(user.id);
    if target != user.id && !matches!(user.role, UserRole::Admin | UserRole::ComplianceOfficer) {
        return Err(AppError::Forbidden(
            "You can only view your own KYC status".to_string(),
        ));
    }
    get_kyc_status_internal(&state, target).await
}

async fn get_kyc_status_internal(
    state: &crate::AppState,
    user_id: Uuid,
) -> AppResult<Json<KycStatusResponse>> {
    let kyc = queries::get_kyc_status(&state.db, user_id).await?;
    let uploaded_documents = queries::get_kyc_documents(&state.db, kyc.id).await?;

    let required_documents = vec![
        DocumentRequirement {
            document_type: DocumentType::Passport,
            required: true,
            description: "Government-issued passport or national ID".to_string(),
            uploaded: uploaded_documents.iter().any(|d| {
                matches!(d.document_type, DocumentType::Passport | DocumentType::NationalId)
            }),
        },
        DocumentRequirement {
            document_type: DocumentType::UtilityBill,
            required: true,
            description: "Proof of address (utility bill or bank statement)".to_string(),
            uploaded: uploaded_documents.iter().any(|d| {
                matches!(d.document_type, DocumentType::UtilityBill | DocumentType::BankStatement)
            }),
        },
    ];

    Ok(Json(KycStatusResponse {
        kyc_id: kyc.id,
        user_id: kyc.user_id,
        verification_status: kyc.verification_status,
        risk_level: kyc.risk_level,
        documents_verified: kyc.documents_verified,
        identity_verified: kyc.identity_verified,
        address_verified: kyc.address_verified,
        phone_verified: kyc.phone_verified,
        email_verified: kyc.email_verified,
        verification_score: kyc.verification_score,
        verification_date: kyc.verification_date,
        expiry_date: kyc.expiry_date,
        rejection_reason: kyc.rejection_reason,
        required_documents,
        uploaded_documents,
    }))
}

pub async fn upload_document(
    State(state): State<crate::AppState>,
    Extension(user): Extension<User>,
    Path(kyc_id): Path<Uuid>,
    mut multipart: Multipart,
) -> AppResult<Json<DocumentUploadResponse>> {
    let uploaded = queries::upload_document(&state.db, &mut multipart, user.id, kyc_id).await?;
    Ok(Json(uploaded))
}

