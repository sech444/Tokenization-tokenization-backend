// src/database/queries.rs

use axum::extract::Multipart;
use sqlx::{PgPool, Row, QueryBuilder, Postgres, FromRow};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::utils::errors::{AppResult, AppError};
use crate::handlers::marketplace::{ ListingFilters, OrderFilters};
use crate::AppState;
use crate::utils::auth::Claims;
use crate::handlers::admin::{KycVerificationSummary, KycListResponse, KycListQuery, UserListQuery, UserSummary};
use crate::models::kyc::{KycVerification, DocumentType, DocumentVerificationStatus, VerificationStatus, RiskLevel, UpdateKycParams, KycListItem };
use crate::handlers::kyc::{DocumentUploadResponse, DocumentSummary};
use crate::handlers::project::CreateProjectRequest;
use crate::models::project::{ProjectResponse, ProjectStatus, Project, ProjectType};

#[derive(Clone, Debug)]
pub struct Database {
    pub pool: PgPool,
}

#[derive(Debug, Clone, FromRow)]
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

#[derive(Debug, Clone, FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
    pub verification_score: Option<Decimal>,
    pub verification_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct KycWithUserInfo {
    kyc_id: Uuid,
    user_id: Uuid,
    verification_status: String,
    risk_level: String,
    kyc_created_at: DateTime<Utc>,
    user_email: String,
    user_first_name: Option<String>,
    user_last_name: Option<String>,
}

// --- Function Implementations ---

pub async fn initiate_kyc(pool: &PgPool, user_id: Uuid) -> AppResult<KycVerification> {
    let existing = sqlx::query_as::<_, KycVerification>(
        "SELECT * FROM kyc_verifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(kyc) = existing {
        if kyc.is_verified() && !kyc.is_expired() {
            return Ok(kyc);
        }
    }

    let new_kyc = KycVerification::new(user_id);
    let created_kyc = sqlx::query_as::<_, KycVerification>(
        r#"
        INSERT INTO kyc_verifications (id, user_id, verification_status, risk_level)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(new_kyc.id)
    .bind(new_kyc.user_id)
    .bind(new_kyc.verification_status)
    .bind(new_kyc.risk_level)
    .fetch_one(pool)
    .await?;

    Ok(created_kyc)
}

pub async fn get_listings_with_details(
    pool: &PgPool,
    filters: &ListingFilters,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<DbListing>, i64)> {
    let mut conditions = QueryBuilder::new(" WHERE 1=1");
    if let Some(token_id) = filters.token_id {
        conditions.push(" AND l.token_id = ");
        conditions.push_bind(token_id);
    }
    if let Some(ref listing_type) = filters.listing_type {
        conditions.push(" AND l.listing_type = ");
        conditions.push_bind(listing_type);
    }
    if let Some(ref min_price) = filters.min_price {
        conditions.push(" AND l.price_per_token >= ");
        conditions.push_bind(min_price);
    }
    if let Some(ref max_price) = filters.max_price {
        conditions.push(" AND l.price_per_token <= ");
        conditions.push_bind(max_price);
    }

    let mut count_query = QueryBuilder::new("SELECT COUNT(*) FROM marketplace_listings l JOIN tokens t ON t.id = l.token_id JOIN users u ON u.id = l.user_id ");
    count_query.push(conditions.sql());
    let total_row = count_query.build().fetch_one(pool).await?;
    let total: i64 = total_row.get(0);

    let mut data_query = QueryBuilder::new("SELECT l.id, l.token_id, t.name as token_name, t.symbol as token_symbol, l.user_id, u.username as seller_username, l.quantity, l.price_per_token, l.listing_type, l.status, l.created_at, l.expires_at FROM marketplace_listings l JOIN tokens t ON t.id = l.token_id JOIN users u ON u.id = l.user_id ");
    data_query.push(conditions.sql());
    data_query.push(" ORDER BY l.created_at DESC LIMIT ");
    data_query.push_bind(limit);
    data_query.push(" OFFSET ");
    data_query.push_bind(offset);

    let listings = data_query.build_query_as().fetch_all(pool).await?;
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
    let listing = sqlx::query_as(
        r#"
        INSERT INTO marketplace_listings (user_id, token_id, quantity, price_per_token, listing_type, status, expires_at)
        VALUES ($1, $2, $3, $4, $5, 'active', $6)
        RETURNING id, token_id, NULL as token_name, NULL as token_symbol, user_id, NULL as seller_username, quantity, price_per_token, listing_type, status, created_at, expires_at
        "#,
    )
    .bind(user_id).bind(token_id).bind(quantity).bind(price_per_token).bind(listing_type).bind(expires_at)
    .fetch_one(pool).await?;
    Ok(listing)
}

pub async fn get_listing_by_id(pool: &PgPool, listing_id: Uuid) -> AppResult<Option<DbListing>> {
    let listing = sqlx::query_as::<_, DbListing>(
        r#"
        SELECT l.id, l.token_id, t.name as token_name, t.symbol as token_symbol, l.user_id, u.username as seller_username, l.quantity, l.price_per_token, l.listing_type, l.status, l.created_at, l.expires_at
        FROM marketplace_listings l JOIN tokens t ON t.id = l.token_id JOIN users u ON u.id = l.user_id
        WHERE l.id = $1
        "#,
    ).bind(listing_id).fetch_optional(pool).await?;
    Ok(listing)
}

pub async fn get_user_token_balance(pool: &PgPool, user_id: Uuid, token_id: Uuid) -> AppResult<i64> {
    let row = sqlx::query("SELECT balance FROM user_token_balances WHERE user_id = $1 AND token_id = $2").bind(user_id).bind(token_id).fetch_optional(pool).await?;
    Ok(row.map(|r| r.get("balance")).unwrap_or(0))
}

pub async fn execute_trade(pool: &PgPool, listing_id: Uuid, buyer_id: Uuid, quantity: i64, total_cost: Decimal) -> AppResult<DbOrder> {
    let mut tx = pool.begin().await?;
    let listing = sqlx::query_as::<_, DbListing>("SELECT * FROM marketplace_listings WHERE id = $1 FOR UPDATE").bind(listing_id).fetch_one(&mut *tx).await?;
    if listing.quantity < quantity { return Err(AppError::ValidationError("Insufficient listing quantity".to_string())); }
    sqlx::query("UPDATE marketplace_listings SET quantity = quantity - $1, status = CASE WHEN quantity - $1 = 0 THEN 'filled' ELSE status END WHERE id = $2").bind(quantity).bind(listing_id).execute(&mut *tx).await?;
    let order = sqlx::query_as::<_, DbOrder>(
        r#"
        INSERT INTO marketplace_orders (listing_id, buyer_id, seller_id, token_id, quantity, price_per_token, total_amount, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'completed')
        RETURNING id, listing_id, buyer_id, seller_id, token_id, NULL as token_name, quantity, price_per_token, total_amount, status, created_at, completed_at
        "#,
    ).bind(listing_id).bind(buyer_id).bind(listing.user_id).bind(listing.token_id).bind(quantity).bind(listing.price_per_token).bind(total_cost).fetch_one(&mut *tx).await?;
    tx.commit().await?;
    Ok(order)
}

pub async fn get_orders_with_details(pool: &PgPool, filters: &OrderFilters, limit: i64, offset: i64) -> AppResult<(Vec<DbOrder>, i64)> {
    let mut conditions = QueryBuilder::new(" WHERE 1=1");
    if let Some(user_id) = filters.user_id {
        conditions.push(" AND (o.buyer_id = ");
        conditions.push_bind(user_id);
        conditions.push(" OR o.seller_id = ");
        conditions.push_bind(user_id);
        conditions.push(")");
    }
    if let Some(ref status) = filters.status {
        conditions.push(" AND o.status = ");
        conditions.push_bind(status);
    }
    let mut count_query = QueryBuilder::new("SELECT COUNT(*) FROM marketplace_orders o LEFT JOIN tokens t ON t.id = o.token_id ");
    count_query.push(conditions.sql());
    let total_row = count_query.build().fetch_one(pool).await?;
    let total: i64 = total_row.get(0);
    let mut data_query = QueryBuilder::new("SELECT o.*, t.name as token_name FROM marketplace_orders o LEFT JOIN tokens t ON t.id = o.token_id ");
    data_query.push(conditions.sql());
    data_query.push(" ORDER BY o.created_at DESC LIMIT ");
    data_query.push_bind(limit);
    data_query.push(" OFFSET ");
    data_query.push_bind(offset);
    let orders = data_query.build_query_as().fetch_all(pool).await?;
    Ok((orders, total))
}

pub async fn create_project(pool: &PgPool, claims: Claims, payload: &CreateProjectRequest) -> Result<ProjectResponse, AppError> {
    if payload.name.is_empty() || payload.description.is_empty() { return Err(AppError::BadRequest("Project name and description are required".into())); }
    if payload.total_value <= 0 { return Err(AppError::BadRequest("Total value must be positive".into())); }
    if payload.minimum_investment <= 0 { return Err(AppError::BadRequest("Minimum investment must be positive".into())); }
    if let Some(max_inv) = payload.maximum_investment { if max_inv < payload.minimum_investment { return Err(AppError::BadRequest("Maximum investment cannot be less than minimum investment".into())); } }
    let valid_types = ["residential", "commercial", "industrial", "mixed_use", "land", "hospitality"];
    if !valid_types.contains(&payload.project_type.to_string().as_str()) { return Err(AppError::BadRequest("Invalid project type".into())); }
    if payload.investment_period_months <= 0 { return Err(AppError::BadRequest("Investment period must be positive".into())); }
    let project = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (name, description, project_type, owner_id, location, property_address, total_value, minimum_investment, maximum_investment, expected_return, investment_period_months, property_details, legal_documents, images, kyc_required)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) RETURNING *
        "#,
    ).bind(&payload.name).bind(&payload.description).bind(payload.project_type).bind(claims.user_id).bind(&payload.location).bind(&payload.property_address).bind(payload.total_value).bind(payload.minimum_investment).bind(payload.maximum_investment).bind(payload.expected_return).bind(payload.investment_period_months).bind(&payload.property_details).bind(&payload.legal_documents).bind(&payload.images).bind(payload.kyc_required).fetch_one(pool).await?;
    Ok(project.into())
}

pub async fn get_project(pool: &PgPool, project_id: Uuid) -> Result<Option<ProjectResponse>, AppError> {
    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1").bind(project_id).fetch_optional(pool).await?;
    Ok(project.map(|p| p.into()))
}

pub async fn fetch_users(pool: &PgPool, _query: &UserListQuery, page: u32, limit: u32) -> Result<(Vec<UserSummary>, i64), AppError> {
    let offset = (page - 1) * limit;
    let rows = sqlx::query_as::<_, UserSummary>("SELECT id, email, status::text as status, last_login, role::text as role, is_active, created_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(limit as i64).bind(offset as i64).fetch_all(pool).await?;
    let total_row = sqlx::query("SELECT COUNT(*) FROM users").fetch_one(pool).await?;
    let total_count: i64 = total_row.get(0);
    Ok((rows, total_count))
}

pub async fn count_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(pool).await?;
    Ok(row.0)
}

pub async fn count_active_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE is_active = TRUE").fetch_one(pool).await?;
    Ok(row.0)
}

pub async fn count_projects(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects").fetch_one(pool).await?;
    Ok(row.0)
}

pub async fn count_active_projects(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM projects WHERE status = 'active'").fetch_one(pool).await?;
    Ok(row.0)
}

pub async fn list_kyc_verifications(pool: &PgPool, query: &KycListQuery, page: u32, limit: u32) -> AppResult<(Vec<KycVerificationSummary>, i64)> {
    let offset = (page.saturating_sub(1) * limit) as i64;
    let mut conditions = vec!["1=1".to_string()];
    if query.status.is_some() { conditions.push("verification_status = $1".to_string()); }
    if query.risk_level.is_some() { conditions.push("risk_level = $2".to_string()); }
    let where_clause = conditions.join(" AND ");
    let count_sql = format!("SELECT COUNT(*) FROM kyc_verifications WHERE {}", where_clause);
    let mut count_query = sqlx::query(&count_sql);
    if let Some(status) = &query.status { count_query = count_query.bind(status.to_string()); }
    if let Some(risk) = &query.risk_level { count_query = count_query.bind(risk.to_string()); }
    let total_row = count_query.fetch_one(pool).await?;
    let total_count: i64 = total_row.get(0);
    let data_sql = format!("SELECT id, user_id, verification_status::text, risk_level::text, verification_score, created_at, updated_at FROM kyc_verifications WHERE {} ORDER BY created_at DESC LIMIT {} OFFSET {}", where_clause, limit, offset);
    let mut data_query = sqlx::query_as::<_, KycVerificationSummary>(&data_sql);
    if let Some(status) = &query.status { data_query = data_query.bind(status.to_string()); }
    if let Some(risk) = &query.risk_level { data_query = data_query.bind(risk.to_string()); }
    let rows = data_query.fetch_all(pool).await?;
    Ok((rows, total_count))
}

pub async fn update_project_status(pool: &PgPool, project_id: Uuid, status: ProjectStatus) -> Result<(), AppError> {
    sqlx::query("UPDATE projects SET status = $1, updated_at = NOW() WHERE id = $2").bind(status).bind(project_id).execute(pool).await?;
    Ok(())
}

pub async fn list_kyc(pool: &PgPool, query: KycListQuery) -> AppResult<KycListResponse> {
    let (verifications, total_count) = list_kyc_verifications(pool, &query, query.page.unwrap_or(1), query.limit.unwrap_or(20)).await?;
    let total_pages = ((total_count as f64) / (query.limit.unwrap_or(20) as f64)).ceil() as u32;
    Ok(KycListResponse { verifications, total_count: total_count as u32, page: query.page.unwrap_or(1), limit: query.limit.unwrap_or(20), total_pages })
}

pub async fn upload_document(pool: &PgPool, multipart: &mut Multipart, user_id: Uuid, kyc_id: Uuid) -> AppResult<DocumentUploadResponse> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    let mut file_bytes = Vec::new();
    let mut document_type = DocumentType::Passport;
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(format!("Invalid multipart data: {}", e)))? {
        let name = field.name().unwrap_or("").to_string();
        if name == "document_type" {
            let value = field.text().await.map_err(|e| AppError::BadRequest(format!("Invalid document type: {}", e)))?;
            document_type = value.parse().map_err(|_| AppError::ValidationError(format!("Invalid document type: {}", value)))?;
        } else if name == "file" {
            file_bytes = field.bytes().await.map_err(|e| AppError::BadRequest(format!("Invalid file data: {}", e)))?.to_vec();
        }
    }
    let file_id = Uuid::new_v4();
    let path = format!("uploads/{}.bin", file_id);
    let mut file = File::create(&path).await?;
    file.write_all(&file_bytes).await?;
    sqlx::query("INSERT INTO kyc_documents (id, kyc_id, user_id, document_type, verification_status, file_path) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(file_id).bind(kyc_id).bind(user_id).bind(&document_type).bind(DocumentVerificationStatus::Pending).bind(path)
        .execute(pool).await?;
    Ok(DocumentUploadResponse { document_id: file_id, document_type, upload_status: "uploaded".to_string(), verification_status: DocumentVerificationStatus::Pending, confidence_score: None })
}

pub async fn get_kyc_status(pool: &PgPool, user_id: Uuid) -> AppResult<KycRecord> {
    let record = sqlx::query_as::<_, KycRecord>("SELECT * FROM kyc_verifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1").bind(user_id).fetch_one(pool).await?;
    Ok(record)
}

pub async fn get_kyc_documents(pool: &PgPool, kyc_id: Uuid) -> AppResult<Vec<DocumentSummary>> {
    let documents = sqlx::query_as::<_, DocumentSummary>("SELECT id, document_type::text, verification_status::text, uploaded_at FROM kyc_documents WHERE kyc_id = $1 ORDER BY uploaded_at DESC").bind(kyc_id).fetch_all(pool).await?;
    Ok(documents)
}

pub async fn update_kyc_status(pool: &PgPool, params: UpdateKycParams) -> AppResult<KycRecord> {
    let verification_status = if params.approved { VerificationStatus::Approved } else { VerificationStatus::Rejected };
    let record = sqlx::query_as::<_, KycRecord>(
        r#"
        UPDATE kyc_verifications SET verification_status = $2, verification_date = CASE WHEN $2 = 'approved' THEN NOW() ELSE verification_date END,
        expiry_date = CASE WHEN $2 = 'approved' THEN NOW() + INTERVAL '1 year' ELSE expiry_date END,
        rejection_reason = CASE WHEN $2 = 'rejected' THEN $3 ELSE rejection_reason END, notes = $3, approved_by = $4, approved_at = NOW(), updated_at = NOW()
        WHERE user_id = $1 AND id = (SELECT id FROM kyc_verifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1)
        RETURNING *
        "#,
    ).bind(params.user_id).bind(verification_status).bind(params.notes).bind(params.approved_by).fetch_one(pool).await?;
    Ok(record)
}

pub async fn update_kyc_status_simple(pool: &PgPool, params: UpdateKycParams) -> AppResult<()> {
    let verification_status = if params.approved { VerificationStatus::Approved } else { VerificationStatus::Rejected };
    let result = sqlx::query(
        r#"
        UPDATE kyc_verifications SET verification_status = $2, rejection_reason = CASE WHEN $2 = 'rejected' THEN $3 ELSE rejection_reason END, updated_at = NOW()
        WHERE user_id = $1 AND id = (SELECT id FROM kyc_verifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1)
        "#,
    ).bind(params.user_id).bind(verification_status).bind(params.notes).bind(params.approved_by).execute(pool).await?;
    if result.rows_affected() == 0 { return Err(AppError::NotFound("KYC record not found".to_string())); }
    Ok(())
}

pub async fn get_pending_kyc(db: &PgPool) -> Result<Vec<KycListItem>, sqlx::Error> {
    sqlx::query_as::<_, KycListItem>("SELECT id, user_id, verification_status, risk_level, created_at FROM kyc_verifications WHERE verification_status = 'pending' ORDER BY created_at DESC").fetch_all(db).await
}

pub async fn get_pending_kyc_with_user_info(pool: &PgPool) -> AppResult<Vec<KycWithUserInfo>> {
    let rows = sqlx::query_as::<_, KycWithUserInfo>(
        r#"
        SELECT k.id as kyc_id, k.user_id, k.verification_status::text, k.risk_level::text, k.created_at as kyc_created_at, u.email as user_email, u.first_name as user_first_name, u.last_name as user_last_name
        FROM kyc_verifications k JOIN users u ON k.user_id = u.id
        WHERE k.verification_status = 'pending' ORDER BY k.created_at ASC
        "#,
    ).fetch_all(pool).await?;
    Ok(rows)
}