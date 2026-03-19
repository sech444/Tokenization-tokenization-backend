// src/handlers/tokenization.rs

use crate::AppState; 
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use uuid::Uuid;

use crate::{
    services::tokenization_service::{TokenizationService, TokenizationParams, TokenizationStatus},
    utils::errors::AppError,
};

#[derive(Debug, Deserialize)]
pub struct TokenizeProjectRequest {
    pub token_name: String,
    pub token_symbol: String,
    pub total_supply: u64,
    pub decimals: u8,
    pub metadata_uri: String,
}

#[derive(Debug, Serialize)]
pub struct TokenizeProjectResponse {
    pub success: bool,
    pub deed_id: u64,
    pub token_address: String,
    pub transaction_hash: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyKYCRequest {
    pub user_address: String,
    pub document_hash: String,
    pub risk_score: u8,
    pub jurisdiction: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyKYCResponse {
    pub success: bool,
    pub transaction_hash: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct KYCStatusResponse {
    pub address: String,
    pub is_verified: bool,
    pub checked_at: String,
}

#[derive(Debug, Serialize)]
pub struct TokenizationStatusResponse {
    pub project_id: String,
    pub status: TokenizationStatus,
    pub checked_at: String,
}

/// POST /api/projects/{id}/tokenize
/// Tokenize a project (convert to on-chain asset + create ERC20 + NFT deed)
// pub async fn tokenize_project(
//     State(state): State<AppState>,
//     Path(project_id): Path<Uuid>,
//     Json(request): Json<TokenizeProjectRequest>,
// ) -> Result<impl IntoResponse, AppError> {
//     info!("Tokenizing project: {}", project_id);
use crate::middleware::auth::AuthenticatedUser;

pub async fn tokenize_project(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,   // ✅ requires login
    Path(project_id): Path<Uuid>,
    Json(request): Json<TokenizeProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("User {} tokenizing project {}", user.id, project_id);

    // Create tokenization service using blockchain service from AppState
    let tokenization_service = TokenizationService::new(
        state.blockchain_service.clone(),
        state.db.clone(),
    );

    let params = TokenizationParams {
        name: request.token_name,
        symbol: request.token_symbol,
        total_supply: request.total_supply,
        decimals: request.decimals,
        metadata_uri: request.metadata_uri,
    };

    match tokenization_service.tokenize_project(project_id, params).await {
        Ok(result) => {
            let response = TokenizeProjectResponse {
                success: true,
                deed_id: result.deed_id,
                token_address: format!("{:?}", result.token_address),
                transaction_hash: result.transaction_hash,
                message: "Project successfully tokenized".to_string(),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to tokenize project {}: {}", project_id, e);
            Err(e)
        }
    }
}

/// POST /api/kyc/verify
/// Verify KYC for a user address
// pub async fn verify_kyc(
//     State(state): State<AppState>,
//     Json(request): Json<VerifyKYCRequest>,
// ) -> Result<impl IntoResponse, AppError> {
//     info!("Verifying KYC for address: {}", request.user_address);

pub async fn verify_kyc(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,   // ✅ logged in
    Json(request): Json<VerifyKYCRequest>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("User {} verifying KYC for {}", user.id, request.user_address);

    let tokenization_service = TokenizationService::new(
        state.blockchain_service.clone(),
        state.db.clone(),
    );

    match tokenization_service
        .verify_user_kyc(
            &request.user_address,
            request.document_hash,
            request.risk_score,
            request.jurisdiction,
        )
        .await
    {
        Ok(tx_hash) => {
            let response = VerifyKYCResponse {
                success: true,
                transaction_hash: tx_hash,
                message: "KYC verified successfully".to_string(),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to verify KYC for {}: {}", request.user_address, e);
            Err(e)
        }
    }
}

/// GET /api/kyc/status/{address}
/// Check KYC status for a user address
pub async fn check_kyc_status(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!("Checking KYC status for address: {}", address);

    let tokenization_service = TokenizationService::new(
        state.blockchain_service.clone(),
        state.db.clone(),
    );

    match tokenization_service.check_kyc_status(&address).await {
        Ok(is_verified) => {
            let response = KYCStatusResponse {
                address,
                is_verified,
                checked_at: chrono::Utc::now().to_rfc3339(),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to check KYC status for {}: {}", address, e);
            Err(e)
        }
    }
}

/// GET /api/projects/{id}/tokenization-status
/// Check tokenization status for a project
pub async fn get_tokenization_status(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    info!("Checking tokenization status for project: {}", project_id);

    let tokenization_service = TokenizationService::new(
        state.blockchain_service.clone(),
        state.db.clone(),
    );

    match tokenization_service.get_tokenization_status(project_id).await {
        Ok(status) => {
            let response = TokenizationStatusResponse {
                project_id: project_id.to_string(),
                status,
                checked_at: chrono::Utc::now().to_rfc3339(),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to check tokenization status for {}: {}", project_id, e);
            Err(e)
        }
    }
}