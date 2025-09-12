// tokenization-backend/src/handlers/token.rs

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

use validator::Validate;
use crate::utils::auth::Claims;
use crate::utils::errors::{AppError, AppResult};
use crate::{
    models::token::{
        BurnTokenRequest, CreateTokenRequest, MintTokenRequest, Token, TokenResponse, TokenStatus,
        TokenType, 
    },
    services::token::TokenService,
    AppState,
};

#[derive(Debug, serde::Deserialize)]
pub struct ListTokensQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub token_type: Option<TokenType>,
    pub project_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
}

#[derive(Debug, serde::Serialize)]
pub struct TokenListResponse {
    pub tokens: Vec<TokenResponse>,
    pub total: i64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct MintResponse {
    pub token_id: Uuid,
    pub amount_minted: i64,
    pub new_total_supply: i64,
    pub new_circulating_supply: i64,
    pub transaction_hash: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct BurnResponse {
    pub token_id: Uuid,
    pub amount_burned: i64,
    pub new_total_supply: i64,
    pub new_circulating_supply: i64,
    pub transaction_hash: Option<String>,
}

/// List all tokens with optional filtering and pagination
pub async fn list_tokens(
    Query(query): Query<ListTokensQuery>,
    State(state): State<AppState>,
) -> AppResult<Json<TokenListResponse>> {
    info!("Listing tokens with query: {:?}", query);

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20).min(100); // cap at 100
    let offset = (page - 1) * limit;

    let token_service = TokenService::new(&state.db);

    let mut filters = HashMap::new();
    if let Some(token_type) = query.token_type {
        filters.insert("token_type".to_string(), json!(token_type));
    }
    if let Some(project_id) = query.project_id {
        filters.insert("project_id".to_string(), json!(project_id));
    }
    if let Some(owner_id) = query.owner_id {
        filters.insert("owner_id".to_string(), json!(owner_id));
    }

    let (tokens, total) = token_service
        .list_tokens_with_filters(filters, limit as i64, offset as i64)
        .await
        .map_err(|e| {
            error!("Failed to list tokens: {}", e);
            AppError::InternalServerError("Failed to fetch tokens".to_string())
        })?;

    let token_responses: Vec<TokenResponse> = tokens.into_iter().map(TokenResponse::from).collect();
    let total_pages = ((total as u64) + limit - 1) / limit;

    Ok(Json(TokenListResponse {
        tokens: token_responses,
        total,
        page,
        limit,
        total_pages,
    }))
}

/// Create a new token
pub async fn create_token(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateTokenRequest>,
) -> AppResult<Json<TokenResponse>> {
    info!(
        "Creating token '{}' for user {}",
        payload.name, claims.user_id
    );

    payload
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation failed: {:?}", e)))?;

    let token_service = TokenService::new(&state.db);

    let new_token = Token {
        id: Uuid::new_v4(),
        project_id: payload.project_id,
        owner_id: claims.user_id,
        name: payload.name,
        symbol: payload.symbol.to_uppercase(),
        description: payload.description,
        token_type: payload.token_type,
        total_supply: payload.total_supply,
        circulating_supply: Some(0),
        decimals: payload.decimals,
        metadata: payload.metadata,
        metadata_uri: payload.metadata_uri,
        compliance_rules: payload.compliance_rules.unwrap_or(json!({})),
        is_active: true,
        initial_price: payload.initial_price,
        current_price: payload.initial_price,
        contract_address: payload.contract_address,
        status: TokenStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created_token = token_service
        .create_token(new_token)
        .await
        .map_err(|e| {
            error!("Failed to create token: {}", e);
            AppError::InternalServerError("Failed to create token".to_string())
        })?;

    Ok(Json(TokenResponse::from(created_token)))
}

/// Get a specific token by ID
pub async fn get_token(
    Path(token_id): Path<Uuid>,
    State(state): State<AppState>,
) -> AppResult<Json<TokenResponse>> {
    info!("Fetching token with ID: {}", token_id);

    let token_service = TokenService::new(&state.db);

    let token = token_service.get_token_by_id(token_id).await.map_err(|e| {
        error!("Failed to fetch token {}: {}", token_id, e);
        AppError::NotFound("Token not found".to_string())
    })?;

    match token {
        Some(token) => Ok(Json(TokenResponse::from(token))),
        None => Err(AppError::NotFound("Token not found".to_string())),
    }
}

/// Mint tokens
pub async fn mint_tokens(
    Path(token_id): Path<Uuid>,
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<MintTokenRequest>,
) -> AppResult<Json<MintResponse>> {
    info!("Minting {} tokens for token {}", payload.amount, token_id);

    if payload.amount <= 0 {
        return Err(AppError::BadRequest("Mint amount must be > 0".to_string()));
    }

    let token_service = TokenService::new(&state.db);

    let token = token_service
        .get_token_by_id(token_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch token {}: {}", token_id, e);
            AppError::InternalServerError("Database error".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Token not found".to_string()))?;

    if token.owner_id != claims.user_id && !claims.is_admin() {
        return Err(AppError::Forbidden(
            "Not allowed to mint this token".to_string(),
        ));
    }

    if !token.is_active {
        return Err(AppError::BadRequest("Token is not active".to_string()));
    }

    let mint_result = token_service
        .mint_tokens(token_id, payload, claims.user_id)
        .await
        .map_err(|e| {
            error!("Failed to mint tokens: {}", e);
            AppError::InternalServerError("Failed to mint tokens".to_string())
        })?;

    Ok(Json(MintResponse {
        token_id,
        amount_minted: mint_result.amount,
        new_total_supply: mint_result.new_total_supply,
        new_circulating_supply: mint_result.new_circulating_supply,
        transaction_hash: mint_result.transaction_hash,
    }))
}

/// Burn tokens
pub async fn burn_tokens(
    Path(token_id): Path<Uuid>,
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<BurnTokenRequest>,
) -> AppResult<Json<BurnResponse>> {
    info!("Burning {} tokens for token {}", payload.amount, token_id);

    if payload.amount <= 0 {
        return Err(AppError::BadRequest("Burn amount must be > 0".to_string()));
    }

    let token_service = TokenService::new(&state.db);

    let token = token_service
        .get_token_by_id(token_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch token {}: {}", token_id, e);
            AppError::InternalServerError("Database error".to_string())
        })?
        .ok_or_else(|| AppError::NotFound("Token not found".to_string()))?;

    if token.owner_id != claims.user_id && !claims.is_admin() {
        return Err(AppError::Forbidden(
            "Not allowed to burn this token".to_string(),
        ));
    }

    if !token.is_active {
        return Err(AppError::BadRequest("Token is not active".to_string()));
    }

    if payload.amount > token.circulating_supply.unwrap_or(0) {
        return Err(AppError::BadRequest(
            "Cannot burn more than circulating supply".to_string(),
        ));
    }

    let burn_result = token_service
        .burn_tokens(token_id, payload, claims.user_id)
        .await
        .map_err(|e| {
            error!("Failed to burn tokens: {}", e);
            AppError::InternalServerError("Failed to burn tokens".to_string())
        })?;

    Ok(Json(BurnResponse {
        token_id,
        amount_burned: burn_result.amount,
        new_total_supply: burn_result.new_total_supply,
        new_circulating_supply: burn_result.new_circulating_supply,
        transaction_hash: burn_result.transaction_hash,
    }))
}
