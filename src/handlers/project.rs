// src/handlers/project.rs

use axum::{
    extract::{Path, Query, State, Json, Extension},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;
use tracing::info;
use std::sync::Arc;
use ethers::prelude::*;

use crate::{
    AppState,
    utils::errors::{AppResult, AppError},
    models::{
        project::{Project, ProjectStatus, ProjectType},
        user::{User, UserRole},
        token::TokenType,
    },
    database::projects as db_projects,
    contracts::{
        hybrid_asset_tokenizer::{HybridAssetTokenizer, AssetRegisteredFilter},
        token_factory::{TokenFactory, TokenCreatedFilter},
    },
};

// --- DTOs ---
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: String,
    pub project_type: ProjectType,
    pub location: Option<String>,
    pub property_address: Option<String>,
    pub total_value: i64,
    pub minimum_investment: i64,
    pub maximum_investment: Option<i64>,
    pub expected_return: Option<Decimal>,
    pub investment_period_months: i32,
    pub property_details: Option<serde_json::Value>,
    pub legal_documents: Option<Vec<String>>,
    pub images: Option<Vec<String>>,
    pub kyc_required: bool,
}

#[derive(Debug, Deserialize)]
pub struct ProjectFilters {
    pub project_type: Option<ProjectType>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub location: Option<String>,
    pub is_tokenized: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct TokenizeProjectRequest {
    pub token_name: String,
    pub token_symbol: String,
    pub total_supply: U256,
    pub decimals: u8,
}

// --- Handlers ---
pub async fn tokenize_project(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<TokenizeProjectRequest>,
) -> AppResult<Json<serde_json::Value>> {
    info!("User {} attempting to tokenize project {}", current_user.id, project_id);

    let project = db_projects::get_project_by_id(&state.db, &project_id)
        .await?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    if project.owner_id != current_user.id && !matches!(current_user.role, UserRole::Admin) {
        return Err(AppError::forbidden("You don't have permission to tokenize this project"));
    }
    if project.is_tokenized {
        return Err(AppError::bad_request("Project is already tokenized"));
    }
    if project.status != ProjectStatus::Approved {
        return Err(AppError::bad_request("Project must be in 'Approved' status to be tokenized"));
    }

    let wallet: LocalWallet = state.config.blockchain.deployer_private_key.parse::<LocalWallet>()?
        .with_chain_id(state.provider.get_chainid().await?.as_u64());
    let client = Arc::new(SignerMiddleware::new(Arc::clone(&state.provider), wallet));

    info!("Step 3: Registering asset on HybridAssetTokenizer for project ID: {}", project_id);
    let tokenizer_contract = HybridAssetTokenizer::new(state.config.blockchain.hybrid_asset_tokenizer_proxy_address, Arc::clone(&client));
    
    let asset_type: u8 = 0;
    let document_hashes: Vec<String> = project.legal_documents.clone().unwrap_or_default();
    let location = project.location.clone().unwrap_or_default();

    let tx_call = tokenizer_contract.register_asset(
        project.name.clone(),
        project.description.clone(),
        asset_type,
        document_hashes,
        location,
    );

    let pending_tx = tx_call.send().await.map_err(|e| AppError::ContractError(format!("Failed to send registerAsset tx: {}", e)))?;
    let receipt = pending_tx.await?.ok_or_else(|| AppError::ContractError("Failed to get receipt for asset registration".to_string()))?;

    let asset_id = receipt.logs.iter().find_map(|log| {
        tokenizer_contract.decode_event::<AssetRegisteredFilter>("AssetRegistered", log.topics.clone(), log.data.clone()).ok().map(|event| event.asset_id)
    }).ok_or_else(|| AppError::ContractError("Could not find AssetRegistered event".to_string()))?;

    info!("Asset successfully registered on-chain with ID: {}", asset_id);

    info!("Step 4: Creating token on TokenFactory for asset ID: {}", asset_id);
    let factory_contract = TokenFactory::new(state.config.blockchain.token_factory_proxy_address, Arc::clone(&client));

    let token_type = TokenType::Security;
    let metadata_uri = format!("https://api.yourplatform.com/tokens/metadata/{}", payload.token_symbol);

    let create_token_call = factory_contract.create_token(
        payload.token_name,
        payload.token_symbol,
        payload.total_supply,
        payload.decimals,
        token_type as u8,
        metadata_uri,
    );

    let pending_token_tx = create_token_call.send().await.map_err(|e| AppError::ContractError(format!("Failed to send createToken tx: {}", e)))?;
    let token_receipt = pending_token_tx.await?.ok_or_else(|| AppError::ContractError("Failed to get receipt for token creation".to_string()))?;

    let new_token_address = token_receipt.logs.iter().find_map(|log| {
        factory_contract.decode_event::<TokenCreatedFilter>("TokenCreated", log.topics.clone(), log.data.clone()).ok().map(|event| event.token_address)
    }).ok_or_else(|| AppError::ContractError("Could not find TokenCreated event".to_string()))?;

    info!("Token successfully created at address: {:?}", new_token_address);

    info!("Step 5: Updating database for project ID: {}", project_id);
    let token_address_str = format!("{:#x}", new_token_address);
    db_projects::update_project_tokenization(&state.db, project_id, &token_address_str).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Project successfully tokenized.",
        "projectId": project_id,
        "onChainAssetId": asset_id.to_string(),
        "tokenContractAddress": token_address_str,
        "transactionHash": token_receipt.transaction_hash,
    })))
}

pub async fn delete_project(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(project_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let project = db_projects::get_project_by_id(&state.db, &project_id)
        .await?
        .ok_or_else(|| AppError::not_found("Project not found"))?;

    if project.owner_id != user.id && !matches!(user.role, UserRole::Admin) {
        return Err(AppError::forbidden("You can only delete your own projects"));
    }
    if matches!(project.status, ProjectStatus::Active | ProjectStatus::Funded) {
        return Err(AppError::bad_request("Cannot delete active or funded projects"));
    }
    if project.is_tokenized {
        return Err(AppError::bad_request("Cannot delete tokenized projects"));
    }

    db_projects::delete_project_by_id(&state.db, project_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ===== THIS IS THE FIX =====
pub async fn list_projects(
    State(state): State<AppState>,
    Query(filters): Query<ProjectFilters>,
) -> Result<impl IntoResponse, AppError> {
    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    // The database function only returns a Vec<Project>, not the total count.
    // We call the count function separately.
    let status_filter = filters.status.and_then(|s| s.parse::<ProjectStatus>().ok());
    let projects = db_projects::list_projects_filtered(&state.db, status_filter, filters.project_type, limit, offset).await?;
    let total = db_projects::count_projects(&state.db).await?; // Call count function
    let total_pages = (total + limit - 1) / limit;

    Ok(Json(serde_json::json!({
        "data": projects,
        "page": page,
        "limit": limit,
        "total": total,
        "total_pages": total_pages
    })))
}

pub async fn create_project(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    let project = db_projects::create_project(&state.db, &payload, user.id).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let project = db_projects::get_project(&state.db, project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".into()))?;

    Ok(Json(project))
}