// tokenization-backend/src/handlers/marketplace.rs


use crate::database::{ queries };

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    utils::auth::Claims,
    utils::errors::AppError,
    AppState,
};

// ─── Request DTOs ───────────────────────────────────────────
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateListingRequest {
    pub token_id: Uuid,
    pub quantity: i64,
    pub price_per_token: rust_decimal::Decimal,
    pub listing_type: String, // "sell" or "buy"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuyTokensRequest {
    pub listing_id: Uuid,
    pub quantity: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListingFilters {
    pub token_id: Option<Uuid>,
    pub listing_type: Option<String>,
    pub min_price: Option<rust_decimal::Decimal>,
    pub max_price: Option<rust_decimal::Decimal>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderFilters {
    pub user_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

// ─── Response DTOs ──────────────────────────────────────────
#[derive(Debug, Serialize)]
pub struct ListingResponse {
    pub id: Uuid,
    pub token_id: Uuid,
    pub token_name: String,
    pub token_symbol: String,
    pub seller_id: Uuid,
    pub seller_username: String,
    pub quantity: i64,
    pub price_per_token: rust_decimal::Decimal,
    pub total_value: rust_decimal::Decimal,
    pub listing_type: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub token_id: Uuid,
    pub token_name: String,
    pub quantity: i64,
    pub price_per_token: rust_decimal::Decimal,
    pub total_amount: rust_decimal::Decimal,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub total_pages: i64,
}

// ─── Handlers ───────────────────────────────────────────────
pub async fn get_listings(
    State(state): State<AppState>,
    Query(filters): Query<ListingFilters>,
) -> Result<impl IntoResponse, AppError> {
    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    let (listings, total) =
        queries::get_listings_with_details(&state.db, &filters, limit, offset).await?;

    let responses: Vec<ListingResponse> = listings
        .into_iter()
        .map(|l| ListingResponse {
            id: l.id,
            token_id: l.token_id,
            token_name: l.token_name.unwrap_or_default(),
            token_symbol: l.token_symbol.unwrap_or_default(),
            seller_id: l.user_id,
            seller_username: l.seller_username.unwrap_or_default(),
            quantity: l.quantity,
            price_per_token: l.price_per_token,
            total_value: l.price_per_token * rust_decimal::Decimal::from(l.quantity),
            listing_type: l.listing_type,
            status: l.status,
            created_at: l.created_at,
            expires_at: l.expires_at,
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: responses,
        page,
        limit,
        total,
        total_pages: (total as f64 / limit as f64).ceil() as i64,
    }))
}

pub async fn create_listing(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateListingRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.listing_type != "sell" && payload.listing_type != "buy" {
        return Err(AppError::bad_request("Invalid listing type"));
    }
    if payload.quantity <= 0 {
        return Err(AppError::bad_request("Quantity must be positive"));
    }
    if payload.price_per_token <= rust_decimal::Decimal::ZERO {
        return Err(AppError::bad_request("Price must be positive"));
    }

    // Check balance for sell orders
    if payload.listing_type == "sell" {
        let balance =
            queries::get_user_token_balance(&state.db, claims.user_id, payload.token_id).await?;
        if balance < payload.quantity {
            return Err(AppError::InsufficientFunds {
                available: balance,
                required: payload.quantity,
            });
        }
    }

    // Get raw DB listing
    let listing = queries::create_listing(
        &state.db,
        claims.user_id,
        payload.token_id,
        payload.quantity,
        payload.price_per_token,
        payload.listing_type.clone(),
        None,
    )
    .await?;

    // Convert DB model into response DTO
    let response = ListingResponse {
        id: listing.id,
        token_id: listing.token_id,
        token_name: listing.token_name.unwrap_or_default(),
        token_symbol: listing.token_symbol.unwrap_or_default(),
        seller_id: listing.user_id,
        seller_username: listing.seller_username.unwrap_or_default(),
        quantity: listing.quantity,
        price_per_token: listing.price_per_token,
        total_value: listing.price_per_token * rust_decimal::Decimal::from(listing.quantity),
        listing_type: listing.listing_type,
        status: listing.status,
        created_at: listing.created_at,
        expires_at: listing.expires_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn buy_tokens(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<BuyTokensRequest>,
) -> Result<impl IntoResponse, AppError> {
    if payload.quantity <= 0 {
        return Err(AppError::bad_request("Quantity must be positive"));
    }

    let listing = queries::get_listing_by_id(&state.db, payload.listing_id)
        .await?
        .ok_or_else(|| AppError::not_found("Listing not found"))?;

    if listing.status != "active" {
        return Err(AppError::bad_request("Listing is not active"));
    }
    if payload.quantity > listing.quantity {
        return Err(AppError::bad_request("Requested quantity exceeds available"));
    }
    if listing.user_id == claims.user_id {
        return Err(AppError::bad_request("Cannot buy from your own listing"));
    }
    if let Some(exp) = listing.expires_at {
        if Utc::now() > exp {
            return Err(AppError::bad_request("Listing has expired"));
        }
    }

    let total_cost = listing.price_per_token * rust_decimal::Decimal::from(payload.quantity);

    // raw DB order
    let order = queries::execute_trade(
        &state.db,
        payload.listing_id,
        claims.user_id,
        payload.quantity,
        total_cost,
    )
    .await?;

    // convert DbOrder → API response
    let response = OrderResponse {
        id: order.id,
        listing_id: order.listing_id,
        buyer_id: order.buyer_id,
        seller_id: order.seller_id,
        token_id: order.token_id.unwrap_or_default(),
        token_name: order.token_name.unwrap_or_default(),
        quantity: order.quantity,
        price_per_token: order.price_per_token,
        total_amount: order.total_amount,
        status: order.status,
        created_at: order.created_at,
        completed_at: order.completed_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_orders(
    State(state): State<AppState>,
    claims: Claims,
    Query(filters): Query<OrderFilters>,
) -> Result<impl IntoResponse, AppError> {
    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;

    let mut effective_filters = filters;
    if effective_filters.user_id.is_none() {
        effective_filters.user_id = Some(claims.user_id);
    }

    let (orders, total) =
        queries::get_orders_with_details(&state.db, &effective_filters, limit, offset).await?;

    let responses: Vec<OrderResponse> = orders
        .into_iter()
        .map(|o| OrderResponse {
            id: o.id,
            listing_id: o.listing_id,
            buyer_id: o.buyer_id,
            seller_id: o.seller_id,
            token_id: o.token_id.unwrap_or_default(),
            token_name: o.token_name.unwrap_or_default(),
            quantity: o.quantity,
            price_per_token: o.price_per_token,
            total_amount: o.total_amount,
            status: o.status,
            created_at: o.created_at,
            completed_at: o.completed_at,
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: responses,
        page,
        limit,
        total,
        total_pages: (total as f64 / limit as f64).ceil() as i64,
    }))
}


