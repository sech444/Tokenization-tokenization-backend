


// tokenization-backend/src/services/token.rs
use sqlx::types::chrono::Utc;
use uuid::Uuid;
use crate::models::token::{
    Token,
    MintTokenRequest, BurnTokenRequest,
};
use crate::utils::errors::{AppError, AppResult};

pub struct TokenService<'a> {
    db: &'a sqlx::PgPool,
}


impl<'a> TokenService<'a> {
    pub fn new(db: &'a sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn create_token(&self, mut token: Token) -> AppResult<Token> {
        // Set timestamps if not already set
        let now = Utc::now();
        token.created_at = now;
        token.updated_at = now;
        
        let created = sqlx::query_as::<_, Token>(
            r#"
            INSERT INTO tokens (
                id, project_id, owner_id, name, symbol, description, token_type,
                total_supply, circulating_supply, decimals, metadata, metadata_uri,
                compliance_rules, is_active, initial_price, current_price,
                contract_address, status, created_at, updated_at
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
            RETURNING *
            "#
        )
        .bind(token.id)
        .bind(token.project_id)
        .bind(token.owner_id)
        .bind(token.name)
        .bind(token.symbol)
        .bind(token.description)
        .bind(token.token_type)
        .bind(token.total_supply)
        .bind(token.circulating_supply)
        .bind(token.decimals)
        .bind(token.metadata)
        .bind(token.metadata_uri)
        .bind(token.compliance_rules)
        .bind(token.is_active)
        .bind(token.initial_price)
        .bind(token.current_price)
        .bind(token.contract_address)
        .bind(token.status)
        .bind(token.created_at)
        .bind(token.updated_at)
        .fetch_one(self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(created)
    }

    pub async fn get_token_by_id(&self, id: Uuid) -> AppResult<Option<Token>> {
        let token = sqlx::query_as::<_, Token>("SELECT * FROM tokens WHERE id = $1")
            .bind(id)
            .fetch_optional(self.db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        Ok(token)
    }

    pub async fn list_tokens_with_filters(
        &self,
        _filters: std::collections::HashMap<String, serde_json::Value>,
        limit: i64,
        offset: i64,
    ) -> AppResult<(Vec<Token>, i64)> {
        // TODO: dynamically apply filters
        let tokens = sqlx::query_as::<_, Token>(
            "SELECT * FROM tokens ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tokens")
            .fetch_one(self.db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok((tokens, total.0))
    }

    pub async fn mint_tokens(
        &self,
        token_id: Uuid,
        request: MintTokenRequest,
        _minter_id: Uuid,
    ) -> AppResult<MintResult> {
        // Validate amount
        if request.amount <= 0 {
            return Err(AppError::BadRequest("Amount must be positive".to_string()));
        }

        let result = sqlx::query!(
            r#"
            UPDATE tokens
            SET circulating_supply = COALESCE(circulating_supply, 0) + $1,
                updated_at = NOW()
            WHERE id = $2 AND is_active = true
            RETURNING total_supply, circulating_supply
            "#,
            request.amount,
            token_id
        )
        .fetch_one(self.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound("Token not found or inactive".to_string()),
            _ => AppError::InternalServerError(e.to_string()),
        })?;

        Ok(MintResult {
            amount: request.amount,
            new_total_supply: result.total_supply,
            new_circulating_supply: result.circulating_supply.unwrap_or(0),
            transaction_hash: None,
        })
    }

    pub async fn burn_tokens(
        &self,
        token_id: Uuid,
        request: BurnTokenRequest,
        _burner_id: Uuid,
    ) -> AppResult<BurnResult> {
        // Validate amount
        if request.amount <= 0 {
            return Err(AppError::BadRequest("Amount must be positive".to_string()));
        }

        let result = sqlx::query!(
            r#"
            UPDATE tokens
            SET circulating_supply = GREATEST(COALESCE(circulating_supply, 0) - $1, 0),
                updated_at = NOW()
            WHERE id = $2 
              AND is_active = true
              AND COALESCE(circulating_supply, 0) >= $1
            RETURNING total_supply, circulating_supply
            "#,
            request.amount,
            token_id
        )
        .fetch_one(self.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::BadRequest(
                "Token not found, inactive, or insufficient circulating supply".to_string()
            ),
            _ => AppError::InternalServerError(e.to_string()),
        })?;

        Ok(BurnResult {
            amount: request.amount,
            new_total_supply: result.total_supply,
            new_circulating_supply: result.circulating_supply.unwrap_or(0),
            transaction_hash: None,
        })
    }
}

/// Return structs for mint/burn
pub struct MintResult {
    pub amount: i64,
    pub new_total_supply: i64,
    pub new_circulating_supply: i64,
    pub transaction_hash: Option<String>,
}

pub struct BurnResult {
    pub amount: i64,
    pub new_total_supply: i64,
    pub new_circulating_supply: i64,
    pub transaction_hash: Option<String>,
}