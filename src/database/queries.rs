

// src/database/queries.rs -  - Version without SQLx macros

use axum::Json;
use axum::{
    extract::{Path, State, Multipart},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::{PgPool, Row, QueryBuilder, Postgres};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::utils::errors::{AppResult, AppError};
use crate::handlers::marketplace::{ ListingFilters, OrderFilters};
use crate::AppState;
use crate::utils::auth::Claims;
use crate::handlers::admin::{KycVerificationSummary, KycListResponse, KycListQuery, UserListQuery, UserSummary};
use crate::models::kyc::{KycVerification, DocumentType, DocumentVerificationStatus, VerificationStatus, RiskLevel, UpdateKycParams };
use crate::handlers::kyc::{DocumentUploadResponse, DocumentSummary};
use crate::handlers::project::CreateProjectRequest;
use crate::models::project::{ProjectResponse, ProjectStatus};
// use crate::models::user::UserResponse;


#[derive(Clone, Debug)]
pub struct Database {
    pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct DbListing {
    pub id: Uuid,
    pub token_id: Uuid,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub user_id: Uuid,
    pub seller_username: Option<String>,
    pub quantity: i64,
    pub price_per_token: Decimal,
    pub listing_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct DbOrder {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    pub token_id: Option<Uuid>,
    pub token_name: Option<String>,
    pub quantity: i64,
    pub price_per_token: Decimal,
    pub total_amount: Decimal,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)] // <-- add Serialize and Deserialize
pub struct KycRecord {
    pub id: Uuid,
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
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Database { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        self.create_tables().await?;
        Ok(())
    }

    async fn create_tables(&self) -> Result<(), sqlx::Error> {
        // Create extensions
        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// KYC functions
pub async fn initiate_kyc(pool: &PgPool, user_id: Uuid) -> AppResult<KycVerification> {
    // Check for existing KYC
    let existing_query = r#"
        SELECT 
            id, user_id, verification_status, risk_level, verification_provider,
            provider_reference_id, documents_verified, identity_verified, 
            address_verified, phone_verified, email_verified, pep_check, 
            sanctions_check, adverse_media_check, verification_score, 
            verification_date, expiry_date, notes, rejection_reason, 
            created_at, updated_at
        FROM kyc_verifications
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 1
    "#;

    if let Some(existing) = sqlx::query(existing_query)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
    {
        let kyc = KycVerification {
            id: existing.get("id"),
            user_id: existing.get("user_id"),
            verification_status: existing.get::<String, _>("verification_status").parse()
                .map_err(|_| AppError::ValidationError("Invalid verification status".to_string()))?,
            risk_level: existing.get::<String, _>("risk_level").parse()
                .map_err(|_| AppError::ValidationError("Invalid risk level".to_string()))?,
            verification_provider: existing.get("verification_provider"),
            provider_reference_id: existing.get("provider_reference_id"),
            documents_verified: existing.get::<Option<bool>, _>("documents_verified").unwrap_or(false),
            identity_verified: existing.get::<Option<bool>, _>("identity_verified").unwrap_or(false),
            address_verified: existing.get::<Option<bool>, _>("address_verified").unwrap_or(false),
            phone_verified: existing.get::<Option<bool>, _>("phone_verified").unwrap_or(false),
            email_verified: existing.get::<Option<bool>, _>("email_verified").unwrap_or(false),
            pep_check: existing.get::<Option<bool>, _>("pep_check").unwrap_or(false),
            sanctions_check: existing.get::<Option<bool>, _>("sanctions_check").unwrap_or(false),
            adverse_media_check: existing.get::<Option<bool>, _>("adverse_media_check").unwrap_or(false),
            verification_score: existing.get("verification_score"),
            verification_date: existing.get("verification_date"),
            expiry_date: existing.get("expiry_date"),
            notes: existing.get("notes"),
            rejection_reason: existing.get("rejection_reason"),
            created_at: existing.get("created_at"),
            updated_at: existing.get("updated_at"),
        };

        if kyc.is_verified() && !kyc.is_expired() {
            return Ok(kyc);
        }
    }

    // Create new KYC
    let new_kyc = KycVerification::new(user_id);
    let create_query = r#"
        INSERT INTO kyc_verifications (
            id, user_id, verification_status, risk_level, verification_provider,
            provider_reference_id, documents_verified, identity_verified, 
            address_verified, phone_verified, email_verified, pep_check, 
            sanctions_check, adverse_media_check, verification_score, 
            verification_date, expiry_date, notes, rejection_reason,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
        RETURNING 
            id, user_id, verification_status, risk_level, verification_provider,
            provider_reference_id, documents_verified, identity_verified, 
            address_verified, phone_verified, email_verified, pep_check, 
            sanctions_check, adverse_media_check, verification_score, 
            verification_date, expiry_date, notes, rejection_reason, 
            created_at, updated_at
    "#;

    let created = sqlx::query(create_query)
        .bind(new_kyc.id)
        .bind(new_kyc.user_id)
        .bind(new_kyc.verification_status.to_string())
        .bind(new_kyc.risk_level.to_string())
        .bind(new_kyc.verification_provider)
        .bind(new_kyc.provider_reference_id)
        .bind(new_kyc.documents_verified)
        .bind(new_kyc.identity_verified)
        .bind(new_kyc.address_verified)
        .bind(new_kyc.phone_verified)
        .bind(new_kyc.email_verified)
        .bind(new_kyc.pep_check)
        .bind(new_kyc.sanctions_check)
        .bind(new_kyc.adverse_media_check)
        .bind(new_kyc.verification_score)
        .bind(new_kyc.verification_date)
        .bind(new_kyc.expiry_date)
        .bind(new_kyc.notes.as_ref())
        .bind(new_kyc.rejection_reason.as_ref())
        .bind(new_kyc.created_at)
        .bind(new_kyc.updated_at)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(KycVerification {
        id: created.get("id"),
        user_id: created.get("user_id"),
        verification_status: created.get::<String, _>("verification_status").parse()
            .map_err(|_| AppError::ValidationError("Invalid verification status".to_string()))?,
        risk_level: created.get::<String, _>("risk_level").parse()
            .map_err(|_| AppError::ValidationError("Invalid risk level".to_string()))?,
        verification_provider: created.get("verification_provider"),
        provider_reference_id: created.get("provider_reference_id"),
        documents_verified: created.get::<Option<bool>, _>("documents_verified").unwrap_or(false),
        identity_verified: created.get::<Option<bool>, _>("identity_verified").unwrap_or(false),
        address_verified: created.get::<Option<bool>, _>("address_verified").unwrap_or(false),
        phone_verified: created.get::<Option<bool>, _>("phone_verified").unwrap_or(false),
        email_verified: created.get::<Option<bool>, _>("email_verified").unwrap_or(false),
        pep_check: created.get::<Option<bool>, _>("pep_check").unwrap_or(false),
        sanctions_check: created.get::<Option<bool>, _>("sanctions_check").unwrap_or(false),
        adverse_media_check: created.get::<Option<bool>, _>("adverse_media_check").unwrap_or(false),
        verification_score: created.get("verification_score"),
        verification_date: created.get("verification_date"),
        expiry_date: created.get("expiry_date"),
        notes: created.get("notes"),
        rejection_reason: created.get("rejection_reason"),
        created_at: created.get("created_at"),
        updated_at: created.get("updated_at"),
    })
}

// Marketplace functions
pub async fn get_listings_with_details(
    pool: &PgPool,
    filters: &ListingFilters,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<DbListing>, i64)> {
    let mut query_builder = QueryBuilder::<Postgres>::new(
        r#"
        SELECT l.id, l.token_id, t.name as token_name, t.symbol as token_symbol,
               l.user_id, u.username as seller_username, l.quantity, l.price_per_token,
               l.listing_type, l.status, l.created_at, l.expires_at
        FROM marketplace_listings l
        JOIN tokens t ON t.id = l.token_id
        JOIN users u ON u.id = l.user_id
        WHERE 1=1
        "#
    );

    if let Some(token_id) = filters.token_id {
        query_builder.push(" AND l.token_id = ");
        query_builder.push_bind(token_id);
    }
    if let Some(ref listing_type) = filters.listing_type {
        query_builder.push(" AND l.listing_type = ");
        query_builder.push_bind(listing_type);
    }
    if let Some(ref min_price) = filters.min_price {
        query_builder.push(" AND l.price_per_token >= ");
        query_builder.push_bind(min_price);
    }
    if let Some(ref max_price) = filters.max_price {
        query_builder.push(" AND l.price_per_token <= ");
        query_builder.push_bind(max_price);
    }

    query_builder.push(" ORDER BY l.created_at DESC LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let query = query_builder.build();
    let rows = query.fetch_all(pool).await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut listings = Vec::new();
    for row in rows {
        listings.push(DbListing {
            id: row.get("id"),
            token_id: row.get("token_id"),
            token_name: row.get("token_name"),
            token_symbol: row.get("token_symbol"),
            user_id: row.get("user_id"),
            seller_username: row.get("seller_username"),
            quantity: row.get("quantity"),
            price_per_token: row.get("price_per_token"),
            listing_type: row.get("listing_type"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            expires_at: row.get("expires_at"),
        });
    }

    // Count query
    let mut count_builder = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(*) as total FROM marketplace_listings l WHERE 1=1"
    );
    
    if let Some(token_id) = filters.token_id {
        count_builder.push(" AND l.token_id = ");
        count_builder.push_bind(token_id);
    }
    if let Some(ref listing_type) = filters.listing_type {
        count_builder.push(" AND l.listing_type = ");
        count_builder.push_bind(listing_type);
    }
    if let Some(ref min_price) = filters.min_price {
        count_builder.push(" AND l.price_per_token >= ");
        count_builder.push_bind(min_price);
    }
    if let Some(ref max_price) = filters.max_price {
        count_builder.push(" AND l.price_per_token <= ");
        count_builder.push_bind(max_price);
    }

    let count_query = count_builder.build();
    let total_row = count_query.fetch_one(pool).await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let total: i64 = total_row.get("total");

    Ok((listings, total))
}

pub async fn create_listing(
    pool: &PgPool,
    user_id: Uuid,
    token_id: Uuid,
    quantity: i64,
    price_per_token: Decimal,
    listing_type: String,
    expires_at: Option<DateTime<Utc>>,
) -> AppResult<DbListing> {
    let query = r#"
        INSERT INTO marketplace_listings
            (id, user_id, token_id, quantity, price_per_token, listing_type, status, created_at, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, 'active', NOW(), $7)
        RETURNING 
            id, token_id, user_id, quantity, price_per_token, listing_type, 
            status, created_at, expires_at
    "#;

    let row = sqlx::query(query)
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(token_id)
        .bind(quantity)
        .bind(price_per_token)
        .bind(listing_type)
        .bind(expires_at)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(DbListing {
        id: row.get("id"),
        token_id: row.get("token_id"),
        token_name: None,
        token_symbol: None,
        user_id: row.get("user_id"),
        seller_username: None,
        quantity: row.get("quantity"),
        price_per_token: row.get("price_per_token"),
        listing_type: row.get("listing_type"),
        status: row.get("status"),
        created_at: row.get("created_at"),
        expires_at: row.get("expires_at"),
    })
}

pub async fn get_listing_by_id(pool: &PgPool, listing_id: Uuid) -> AppResult<Option<DbListing>> {
    let query = r#"
        SELECT l.id, l.token_id, t.name as token_name, 
               t.symbol as token_symbol,
               l.user_id, u.username as seller_username, 
               l.quantity, l.price_per_token,
               l.listing_type, l.status, l.created_at, l.expires_at
        FROM marketplace_listings l
        JOIN tokens t ON t.id = l.token_id
        JOIN users u ON u.id = l.user_id
        WHERE l.id = $1
    "#;

    let row = sqlx::query(query)
        .bind(listing_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    match row {
        Some(r) => Ok(Some(DbListing {
            id: r.get("id"),
            token_id: r.get("token_id"),
            token_name: r.get("token_name"),
            token_symbol: r.get("token_symbol"),
            user_id: r.get("user_id"),
            seller_username: r.get("seller_username"),
            quantity: r.get("quantity"),
            price_per_token: r.get("price_per_token"),
            listing_type: r.get("listing_type"),
            status: r.get("status"),
            created_at: r.get("created_at"),
            expires_at: r.get("expires_at"),
        })),
        None => Ok(None),
    }
}

pub async fn get_user_token_balance(
    pool: &PgPool,
    user_id: Uuid,
    token_id: Uuid,
) -> AppResult<i64> {
    let query = "SELECT COALESCE(SUM(balance), 0) as total FROM user_tokens WHERE user_id = $1 AND token_id = $2";
    
    let row = sqlx::query(query)
        .bind(user_id)
        .bind(token_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let balance = row.map(|r| r.get::<i64, _>("total")).unwrap_or(0);
    Ok(balance)
}

pub async fn execute_trade(
    pool: &PgPool,
    listing_id: Uuid,
    buyer_id: Uuid,
    quantity: i64,
    total_cost: Decimal,
) -> AppResult<DbOrder> {
    let mut tx = pool.begin().await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Lock and get listing
    let listing_query = r#"
        SELECT 
            id, token_id, user_id, quantity, price_per_token, 
            listing_type, status, created_at, expires_at
        FROM marketplace_listings 
        WHERE id = $1 FOR UPDATE
    "#;

    let listing_row = sqlx::query(listing_query)
        .bind(listing_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let listing = DbListing {
        id: listing_row.get("id"),
        token_id: listing_row.get("token_id"),
        token_name: None,
        token_symbol: None,
        user_id: listing_row.get("user_id"),
        seller_username: None,
        quantity: listing_row.get("quantity"),
        price_per_token: listing_row.get("price_per_token"),
        listing_type: listing_row.get("listing_type"),
        status: listing_row.get("status"),
        created_at: listing_row.get("created_at"),
        expires_at: listing_row.get("expires_at"),
    };

    if listing.quantity < quantity {
        return Err(AppError::ValidationError("Insufficient listing quantity".to_string()));
    }

    // Update listing
    let update_query = r#"
        UPDATE marketplace_listings
        SET quantity = quantity - $1,
            status = CASE WHEN quantity - $1 = 0 THEN 'filled' ELSE status END
        WHERE id = $2
    "#;

    sqlx::query(update_query)
        .bind(quantity)
        .bind(listing_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Create order
    let order_query = r#"
        INSERT INTO marketplace_orders
            (id, listing_id, buyer_id, seller_id, token_id,
             quantity, price_per_token, total_amount, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'completed', NOW())
        RETURNING 
            id, listing_id, buyer_id, seller_id, token_id, quantity, 
            price_per_token, total_amount, status, created_at, completed_at
    "#;

    let order_row = sqlx::query(order_query)
        .bind(Uuid::new_v4())
        .bind(listing_id)
        .bind(buyer_id)
        .bind(listing.user_id)
        .bind(listing.token_id)
        .bind(quantity)
        .bind(listing.price_per_token)
        .bind(total_cost)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tx.commit().await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(DbOrder {
        id: order_row.get("id"),
        listing_id: order_row.get("listing_id"),
        buyer_id: order_row.get("buyer_id"),
        seller_id: order_row.get("seller_id"),
        token_id: order_row.get("token_id"),
        token_name: None,
        quantity: order_row.get("quantity"),
        price_per_token: order_row.get("price_per_token"),
        total_amount: order_row.get("total_amount"),
        status: order_row.get("status"),
        created_at: order_row.get("created_at"),
        completed_at: order_row.get("completed_at"),
    })
}

pub async fn get_orders_with_details(
    pool: &PgPool,
    filters: &OrderFilters,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<DbOrder>, i64)> {
    let mut query_builder = QueryBuilder::<Postgres>::new(
        r#"
        SELECT o.id, o.listing_id, o.buyer_id, o.seller_id,
               o.token_id, t.name as token_name, o.quantity, o.price_per_token,
               o.total_amount, o.status, o.created_at, o.completed_at
        FROM marketplace_orders o
        LEFT JOIN tokens t ON t.id = o.token_id
        WHERE 1=1
        "#
    );

    if let Some(user_id) = filters.user_id {
        query_builder.push(" AND (o.buyer_id = ");
        query_builder.push_bind(user_id);
        query_builder.push(" OR o.seller_id = ");
        query_builder.push_bind(user_id);
        query_builder.push(")");
    }
    if let Some(ref status) = filters.status {
        query_builder.push(" AND o.status = ");
        query_builder.push_bind(status);
    }

    query_builder.push(" ORDER BY o.created_at DESC LIMIT ");
    query_builder.push_bind(limit);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let query = query_builder.build();
    let rows = query.fetch_all(pool).await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut orders = Vec::new();
    for row in rows {
        orders.push(DbOrder {
            id: row.get("id"),
            listing_id: row.get("listing_id"),
            buyer_id: row.get("buyer_id"),
            seller_id: row.get("seller_id"),
            token_id: row.get("token_id"),
            token_name: row.get("token_name"),
            quantity: row.get("quantity"),
            price_per_token: row.get("price_per_token"),
            total_amount: row.get("total_amount"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            completed_at: row.get("completed_at"),
        });
    }

    // Count query
    let mut count_builder = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(*) as total FROM marketplace_orders o WHERE 1=1"
    );
    
    if let Some(user_id) = filters.user_id {
        count_builder.push(" AND (o.buyer_id = ");
        count_builder.push_bind(user_id);
        count_builder.push(" OR o.seller_id = ");
        count_builder.push_bind(user_id);
        count_builder.push(")");
    }
    if let Some(ref status) = filters.status {
        count_builder.push(" AND o.status = ");
        count_builder.push_bind(status);
    }

    let count_query = count_builder.build();
    let total_row = count_query.fetch_one(pool).await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let total: i64 = total_row.get("total");

    Ok((orders, total))
}

/// Create a new project
pub async fn create_project(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validation
    if payload.name.is_empty() || payload.description.is_empty() {
        return Err(AppError::BadRequest("Project name and description are required".into()));
    }
    if payload.total_value <= 0 {
        return Err(AppError::BadRequest("Total value must be positive".into()));
    }
    if payload.minimum_investment <= 0 {
        return Err(AppError::BadRequest("Minimum investment must be positive".into()));
    }
    if let Some(max_inv) = payload.maximum_investment {
        if max_inv < payload.minimum_investment {
            return Err(AppError::BadRequest("Maximum investment cannot be less than minimum investment".into()));
        }
    }

    let valid_types = [
        "residential", "commercial", "industrial", "mixed_use", "land", "hospitality",
    ];
    if !valid_types.contains(&payload.project_type.as_str()) {
        return Err(AppError::BadRequest("Invalid project type".into()));
    }

    if payload.investment_period_months <= 0 {
        return Err(AppError::BadRequest("Investment period must be positive".into()));
    }

    // Insert into database
    let row = sqlx::query(
        r#"
        INSERT INTO projects (
            name, description, project_type, owner_id, location, property_address,
            total_value, minimum_investment, maximum_investment, expected_return,
            investment_period_months, property_details, legal_documents, images
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.project_type)
    .bind(claims.user_id)
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
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Map manually into ProjectResponse
    let project = ProjectResponse {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        project_type: row.try_get("project_type")?,
        status: row.try_get("status")?,
        owner_id: row.try_get("owner_id")?,
        location: row.try_get("location")?,
        property_address: row.try_get("property_address")?,
        total_value: row.try_get("total_value")?,
        minimum_investment: row.try_get("minimum_investment")?,
        maximum_investment: row.try_get("maximum_investment")?,
        funds_raised: row.try_get("funds_raised")?,
        investor_count: row.try_get("investor_count")?,
        expected_return: row.try_get("expected_return")?,
        investment_period_months: row.try_get("investment_period_months")?,
        property_details: row.try_get("property_details")?,
        legal_documents: row.try_get("legal_documents")?,
        images: row.try_get("images")?,
        is_tokenized: row.try_get("is_tokenized")?,
        token_contract_address: row.try_get("token_contract_address")?,
        compliance_verified: row.try_get("compliance_verified")?,
        kyc_required: row.try_get("kyc_required")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    };

    Ok((StatusCode::CREATED, Json(project)))
}

/// Get a specific project by ID
pub async fn get_project(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query("SELECT * FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let row = match row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Project not found".into())),
    };

    let project = ProjectResponse {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        project_type: row.try_get("project_type")?,
        status: row.try_get("status")?,
        owner_id: row.try_get("owner_id")?,
        location: row.try_get("location")?,
        property_address: row.try_get("property_address")?,
        total_value: row.try_get("total_value")?,
        minimum_investment: row.try_get("minimum_investment")?,
        maximum_investment: row.try_get("maximum_investment")?,
        funds_raised: row.try_get("funds_raised")?,
        investor_count: row.try_get("investor_count")?,
        expected_return: row.try_get("expected_return")?,
        investment_period_months: row.try_get("investment_period_months")?,
        property_details: row.try_get("property_details")?,
        legal_documents: row.try_get("legal_documents")?,
        images: row.try_get("images")?,
        is_tokenized: row.try_get("is_tokenized")?,
        token_contract_address: row.try_get("token_contract_address")?,
        compliance_verified: row.try_get("compliance_verified")?,
        kyc_required: row.try_get("kyc_required")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    };

    Ok(Json(project))
}




pub async fn list_users(
    pool: &PgPool,
    _query: &UserListQuery,
    page: u32,
    limit: u32,
) -> Result<(Vec<UserSummary>, i64), AppError> {
    let offset = (page - 1) * limit;

    let rows: Vec<UserSummary> = sqlx::query_as!(
        UserSummary,
        r#"
        SELECT 
            id, 
            email,
            status as "status: _", 
            last_login,
            COALESCE(role::text, 'user') as "role!", 
            is_active, 
            created_at
        FROM users
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64,
    )
    .fetch_all(pool)
    .await?;

    let total_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    Ok((rows, total_count.0))
}

/// Count all users
pub async fn count_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Count active users
pub async fn count_active_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE is_active = TRUE")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Count all projects
pub async fn count_projects(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Count active projects
pub async fn count_active_projects(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects WHERE status = 'active'")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn list_kyc_verifications(
    pool: &PgPool,
    query: &KycListQuery,
    page: u32,
    limit: u32,
) -> AppResult<(Vec<KycVerificationSummary>, i64)> {
    let offset = (page.saturating_sub(1) * limit) as i64;

    // Build WHERE clause dynamically
    let mut conditions = vec!["1=1".to_string()];
    if query.status.is_some() {
        conditions.push("verification_status = $1".to_string());
    }
    if query.risk_level.is_some() {
        conditions.push("risk_level = $2".to_string());
    }
    let where_clause = conditions.join(" AND ");

    // Total count
    let total_count: (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM kyc_verifications WHERE {}",
        where_clause
    ))
    .bind(query.status.as_ref())
    .bind(query.risk_level.as_ref())
    .fetch_one(pool)
    .await?;

    // Page of results
    let rows: Vec<KycVerificationSummary> = sqlx::query_as(&format!(
        r#"
        SELECT id, user_id, verification_status, risk_level, verification_score, created_at, updated_at
        FROM kyc_verifications
        WHERE {}
        ORDER BY created_at DESC
        LIMIT {} OFFSET {}
        "#,
        where_clause, limit, offset
    ))
    .bind(query.status.as_ref())
    .bind(query.risk_level.as_ref())
    .fetch_all(pool)
    .await?;

    Ok((rows, total_count.0))
}

pub async fn update_project_status(
    pool: &PgPool,
    project_id: Uuid,
    status: ProjectStatus,
) -> Result<(), AppError> {
    let query = r#"
        UPDATE projects
        SET status = $1, updated_at = NOW()
        WHERE id = $2
    "#;
    
    sqlx::query(query)
        .bind(status.to_string())
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub async fn list_kyc(pool: &PgPool, query: KycListQuery) -> AppResult<KycListResponse> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page.saturating_sub(1) * limit) as i64;

    // Build dynamic conditions
    let mut conditions = vec!["1=1".to_string()];
    let mut args: Vec<(usize, String)> = vec![];
    let mut bind_idx = 1;

    if let Some(status) = query.status {
        conditions.push(format!("verification_status = ${}", bind_idx));
        args.push((bind_idx, status.to_string()));
        bind_idx += 1;
    }
    if let Some(risk) = query.risk_level {
        conditions.push(format!("risk_level = ${}", bind_idx));
        args.push((bind_idx, risk.to_string()));
        bind_idx += 1;
    }
    if let Some(req_review) = query.requires_review {
        conditions.push(format!("requires_review = ${}", bind_idx));
        args.push((bind_idx, req_review.to_string()));
        bind_idx += 1;
    }

    let where_clause = conditions.join(" AND ");

    // Count query
    let total_count: (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM kyc_verifications WHERE {}",
        where_clause
    ))
    .fetch_one(pool)
    .await?;

    // Data query
    let verifications: Vec<KycVerificationSummary> = sqlx::query_as(&format!(
        r#"
        SELECT id, user_id, user_email, verification_status, risk_level, verification_score, created_at, updated_at
        FROM kyc_verifications
        WHERE {}
        ORDER BY created_at DESC
        LIMIT {} OFFSET {}
        "#,
        where_clause, limit, offset
    ))
    .fetch_all(pool)
    .await?;

    let total_pages = ((total_count.0 as f64) / (limit as f64)).ceil() as u32;

    Ok(KycListResponse {
        verifications,
        total_count: total_count.0 as u32,
        page,
        limit,
        total_pages,
    })
}

pub async fn upload_document(
    pool: &PgPool,
    multipart: &mut Multipart,
    user_id: Uuid,
    kyc_id: Uuid,
) -> AppResult<DocumentUploadResponse> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    // Assume only 1 file per upload
    let mut file_bytes = Vec::new();
    let mut document_type = DocumentType::Passport; // default, should parse from form field

    while let Some(field) = multipart.next_field().await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart data: {}", e)))? {
        
        let name = field.name().unwrap_or("").to_string();

        if name == "document_type" {
            let value = field.text().await
                .map_err(|e| AppError::BadRequest(format!("Invalid document type: {}", e)))?;
            document_type = serde_json::from_str(&format!("\"{}\"", value))
                .map_err(|e| AppError::ValidationError(format!("Invalid document type: {}", e)))?;
        } else if name == "file" {
            let data = field.bytes().await
                .map_err(|e| AppError::BadRequest(format!("Invalid file data: {}", e)))?;
            file_bytes.extend_from_slice(&data);
        }
    }

    // Save to disk (replace with S3 if needed)
    let file_id = Uuid::new_v4();
    let path = format!("uploads/{}.bin", file_id);
    let mut file = File::create(&path).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create file: {}", e)))?;
    file.write_all(&file_bytes).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to write file: {}", e)))?;

    // Insert DB record
    let query = r#"
        INSERT INTO kyc_documents (id, kyc_id, user_id, document_type, verification_status, uploaded_at, file_path)
        VALUES ($1, $2, $3, $4, $5, NOW(), $6)
    "#;
    
    sqlx::query(query)
        .bind(file_id)
        .bind(kyc_id)
        .bind(user_id)
        .bind(document_type.to_string())
        .bind(DocumentVerificationStatus::Pending.to_string())
        .bind(path)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(DocumentUploadResponse {
        document_id: file_id,
        document_type,
        upload_status: "uploaded".to_string(),
        verification_status: DocumentVerificationStatus::Pending,
        confidence_score: None,
    })
}

pub async fn get_kyc_status(pool: &PgPool, user_id: Uuid) -> AppResult<KycRecord> { 
    let query = r#"
        SELECT id, user_id, verification_status,
               risk_level,
               documents_verified, identity_verified, address_verified,
               phone_verified, email_verified,
               verification_score, verification_date, expiry_date, rejection_reason
        FROM kyc_verifications
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 1
    "#;
    
    let record = sqlx::query(query)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(KycRecord {
        id: record.get("id"),
        user_id: record.get("user_id"),
        verification_status: record.get::<String, _>("verification_status").parse()
            .map_err(|_| AppError::ValidationError("Invalid verification status".to_string()))?,
        risk_level: record.get::<String, _>("risk_level").parse()
            .map_err(|_| AppError::ValidationError("Invalid risk level".to_string()))?,
        documents_verified: record.get::<Option<bool>, _>("documents_verified").unwrap_or(false),
        identity_verified: record.get::<Option<bool>, _>("identity_verified").unwrap_or(false),
        address_verified: record.get::<Option<bool>, _>("address_verified").unwrap_or(false),
        phone_verified: record.get::<Option<bool>, _>("phone_verified").unwrap_or(false),
        email_verified: record.get::<Option<bool>, _>("email_verified").unwrap_or(false),
        verification_score: record.get("verification_score"),
        verification_date: record.get("verification_date"),
        expiry_date: record.get("expiry_date"),
        rejection_reason: record.get("rejection_reason"),
    })
}

pub async fn get_kyc_documents(pool: &PgPool, kyc_id: Uuid) -> AppResult<Vec<DocumentSummary>> {
    let query = r#"
        SELECT id,
               document_type,
               verification_status,
               uploaded_at
        FROM kyc_documents
        WHERE kyc_id = $1
        ORDER BY uploaded_at DESC
    "#;
    
    let rows = sqlx::query(query)
        .bind(kyc_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut documents = Vec::new();
    for row in rows {
        documents.push(DocumentSummary {
            id: row.get("id"),
            document_type: row.get::<String, _>("document_type").parse()
                .map_err(|_| AppError::ValidationError("Invalid document type".to_string()))?,
            verification_status: row.get::<String, _>("verification_status").parse()
                .map_err(|_| AppError::ValidationError("Invalid verification status".to_string()))?,
            uploaded_at: row.get("uploaded_at"),
        });
    }

    Ok(documents)
}


pub async fn update_kyc_status(
    pool: &PgPool, 
    params: UpdateKycParams
) -> AppResult<KycRecord> {
    let verification_status = if params.approved { "verified" } else { "rejected" };
    
    let query = r#"
        UPDATE kyc_verifications 
        SET 
            verification_status = $2,
            verification_date = CASE 
                WHEN $2 = 'verified' THEN NOW() 
                ELSE verification_date 
            END,
            expiry_date = CASE 
                WHEN $2 = 'verified' THEN NOW() + INTERVAL '1 year'
                ELSE expiry_date 
            END,
            rejection_reason = CASE 
                WHEN $2 = 'rejected' THEN $3
                ELSE rejection_reason
            END,
            approved_by = $4,
            approved_at = NOW(),
            updated_at = NOW()
        WHERE user_id = $1 
        AND id = (
            SELECT id FROM kyc_verifications 
            WHERE user_id = $1 
            ORDER BY created_at DESC 
            LIMIT 1
        )
        RETURNING id, user_id, verification_status, risk_level,
                 documents_verified, identity_verified, address_verified,
                 phone_verified, email_verified, verification_score, 
                 verification_date, expiry_date, rejection_reason
    "#;

    let record = sqlx::query(query)
        .bind(params.user_id)
        .bind(verification_status)
        .bind(params.notes)
        .bind(params.approved_by)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound("KYC record not found".to_string()),
            _ => AppError::DatabaseError(e.to_string())
        })?;

    Ok(KycRecord {
        id: record.get("id"),
        user_id: record.get("user_id"),
        verification_status: record.get::<String, _>("verification_status").parse()
            .map_err(|_| AppError::ValidationError("Invalid verification status".to_string()))?,
        risk_level: record.get::<String, _>("risk_level").parse()
            .map_err(|_| AppError::ValidationError("Invalid risk level".to_string()))?,
        documents_verified: record.get::<Option<bool>, _>("documents_verified").unwrap_or(false),
        identity_verified: record.get::<Option<bool>, _>("identity_verified").unwrap_or(false),
        address_verified: record.get::<Option<bool>, _>("address_verified").unwrap_or(false),
        phone_verified: record.get::<Option<bool>, _>("phone_verified").unwrap_or(false),
        email_verified: record.get::<Option<bool>, _>("email_verified").unwrap_or(false),
        verification_score: record.get("verification_score"),
        verification_date: record.get("verification_date"),
        expiry_date: record.get("expiry_date"),
        rejection_reason: record.get("rejection_reason"),
    })
}

// Alternative version that doesn't return the full record if you don't need it
pub async fn update_kyc_status_simple(
    pool: &PgPool, 
    params: UpdateKycParams
) -> AppResult<()> {
    let verification_status = if params.approved { "verified" } else { "rejected" };
    
    let query = r#"
        UPDATE kyc_verifications 
        SET 
            verification_status = $2,
            verification_date = CASE 
                WHEN $2 = 'verified' THEN NOW() 
                ELSE verification_date 
            END,
            expiry_date = CASE 
                WHEN $2 = 'verified' THEN NOW() + INTERVAL '1 year'
                ELSE expiry_date 
            END,
            rejection_reason = CASE 
                WHEN $2 = 'rejected' THEN $3
                ELSE rejection_reason
            END,
            approved_by = $4,
            approved_at = NOW(),
            updated_at = NOW()
        WHERE user_id = $1 
        AND id = (
            SELECT id FROM kyc_verifications 
            WHERE user_id = $1 
            ORDER BY created_at DESC 
            LIMIT 1
        )
    "#;

    let result = sqlx::query(query)
        .bind(params.user_id)
        .bind(verification_status)
        .bind(params.notes)
        .bind(params.approved_by)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("KYC record not found".to_string()));
    }

    Ok(())
}

pub async fn get_pending_kyc(pool: &PgPool) -> AppResult<Vec<KycRecord>> {
    let query = r#"
        SELECT id, user_id, verification_status, risk_level,
               documents_verified, identity_verified, address_verified,
               phone_verified, email_verified, verification_score, 
               verification_date, expiry_date, rejection_reason,
               created_at, updated_at
        FROM kyc_verifications 
        WHERE verification_status = 'pending'
        ORDER BY created_at ASC
    "#;

    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut pending_kyc = Vec::new();
    
    for row in rows {
        let kyc_record = KycRecord {
            id: row.get("id"),
            user_id: row.get("user_id"),
            verification_status: row.get::<String, _>("verification_status").parse()
                .map_err(|_| AppError::ValidationError("Invalid verification status".to_string()))?,
            risk_level: row.get::<String, _>("risk_level").parse()
                .map_err(|_| AppError::ValidationError("Invalid risk level".to_string()))?,
            documents_verified: row.get::<Option<bool>, _>("documents_verified").unwrap_or(false),
            identity_verified: row.get::<Option<bool>, _>("identity_verified").unwrap_or(false),
            address_verified: row.get::<Option<bool>, _>("address_verified").unwrap_or(false),
            phone_verified: row.get::<Option<bool>, _>("phone_verified").unwrap_or(false),
            email_verified: row.get::<Option<bool>, _>("email_verified").unwrap_or(false),
            verification_score: row.get("verification_score"),
            verification_date: row.get("verification_date"),
            expiry_date: row.get("expiry_date"),
            rejection_reason: row.get("rejection_reason"),
        };
        pending_kyc.push(kyc_record);
    }

    Ok(pending_kyc)
}

// Alternative version with additional user information (more useful for admin review)
pub async fn get_pending_kyc_with_user_info(pool: &PgPool) -> AppResult<Vec<serde_json::Value>> {
    let query = r#"
        SELECT 
            k.id as kyc_id,
            k.user_id,
            k.verification_status,
            k.risk_level,
            k.documents_verified,
            k.identity_verified, 
            k.address_verified,
            k.phone_verified,
            k.email_verified,
            k.verification_score,
            k.verification_date,
            k.expiry_date,
            k.rejection_reason,
            k.created_at as kyc_created_at,
            k.updated_at as kyc_updated_at,
            u.email,
            u.first_name,
            u.last_name,
            u.created_at as user_created_at
        FROM kyc_verifications k
        JOIN users u ON k.user_id = u.id
        WHERE k.verification_status = 'pending'
        ORDER BY k.created_at ASC
    "#;

    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut pending_kyc = Vec::new();
    
    for row in rows {
        let kyc_with_user = serde_json::json!({
            "kyc_id": row.get::<Uuid, _>("kyc_id"),
            "user_id": row.get::<Uuid, _>("user_id"),
            "verification_status": row.get::<String, _>("verification_status"),
            "risk_level": row.get::<String, _>("risk_level"),
            "documents_verified": row.get::<Option<bool>, _>("documents_verified").unwrap_or(false),
            "identity_verified": row.get::<Option<bool>, _>("identity_verified").unwrap_or(false),
            "address_verified": row.get::<Option<bool>, _>("address_verified").unwrap_or(false),
            "phone_verified": row.get::<Option<bool>, _>("phone_verified").unwrap_or(false),
            "email_verified": row.get::<Option<bool>, _>("email_verified").unwrap_or(false),
            "verification_score": row.get::<Option<i32>, _>("verification_score"),
            "verification_date": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("verification_date"),
            "expiry_date": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("expiry_date"),
            "rejection_reason": row.get::<Option<String>, _>("rejection_reason"),
            "kyc_created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("kyc_created_at"),
            "kyc_updated_at": row.get::<chrono::DateTime<chrono::Utc>, _>("kyc_updated_at"),
            "user_email": row.get::<String, _>("email"),
            "user_first_name": row.get::<Option<String>, _>("first_name"),
            "user_last_name": row.get::<Option<String>, _>("last_name"),
            "user_created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("user_created_at")
        });
        pending_kyc.push(kyc_with_user);
    }

    Ok(pending_kyc)
}

// Version that returns count and records separately
pub async fn get_pending_kyc_summary(pool: &PgPool) -> AppResult<(Vec<KycRecord>, usize)> {
    let pending_kyc = get_pending_kyc(pool).await?;
    let count = pending_kyc.len();
    Ok((pending_kyc, count))
}