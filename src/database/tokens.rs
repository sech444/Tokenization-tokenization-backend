

use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::models::{Token};
use crate::utils::errors::AppError;

pub async fn create_token(db: &PgPool, token: &Token) -> Result<Token, AppError> {
    let query = r#"
        INSERT INTO tokens (
            project_id, name, symbol, description, token_type, 
            total_supply, circulating_supply, decimals, owner_id, 
            metadata, is_active, current_price, initial_price, 
            contract_address, status, metadata_uri, compliance_rules
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        RETURNING 
            id, project_id, name, symbol, description, token_type, 
            total_supply, circulating_supply, decimals, owner_id, 
            metadata, is_active, current_price, initial_price, 
            contract_address, status, metadata_uri, compliance_rules, 
            created_at, updated_at
    "#;

    let row = sqlx::query(query)
        .bind(token.project_id)
        .bind(&token.name)
        .bind(&token.symbol)
        .bind(token.description.as_ref())
        .bind(token.token_type.to_string())
        .bind(token.total_supply)
        .bind(token.circulating_supply)
        .bind(token.decimals)
        .bind(token.owner_id)
        .bind(token.metadata.as_ref())
        .bind(token.is_active)
        .bind(token.current_price)
        .bind(token.initial_price)
        .bind(&token.contract_address)
        .bind(token.status.to_string())
        .bind(token.metadata_uri.as_ref())
        .bind(&token.compliance_rules) // Remove as_ref() for JsonValue
        .fetch_one(db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Token {
        id: row.get("id"),
        project_id: row.get("project_id"),
        name: row.get("name"),
        symbol: row.get("symbol"),
        description: row.get("description"),
        token_type: row.get::<String, _>("token_type")
            .parse()
            .map_err(|_| AppError::ValidationError("Invalid token type".to_string()))?,
        total_supply: row.get("total_supply"),
        circulating_supply: row.get("circulating_supply"),
        decimals: row.get("decimals"),
        owner_id: row.get("owner_id"),
        metadata: row.get("metadata"),
        is_active: row.get("is_active"),
        current_price: row.get("current_price"),
        initial_price: row.get("initial_price"),
        contract_address: row.get("contract_address"),
        status: row.get::<String, _>("status")
            .parse()
            .map_err(|_| AppError::ValidationError("Invalid token status".to_string()))?,
        metadata_uri: row.get("metadata_uri"),
        compliance_rules: row.get("compliance_rules"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn get_token_by_id(db: &PgPool, id: &Uuid) -> Result<Option<Token>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            id, project_id, name, symbol, description, token_type, 
            total_supply, circulating_supply, decimals, owner_id, 
            metadata, is_active, current_price, initial_price, 
            contract_address, status, metadata_uri, compliance_rules, 
            created_at, updated_at
        FROM tokens 
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(row.map(|row| -> Result<Token, AppError> {
        let token_type = row.get::<String, _>("token_type")
            .parse()
            .map_err(|_| AppError::ValidationError("Invalid token type".to_string()))?;
        
        let status = row.get::<String, _>("status")
            .parse()
            .map_err(|_| AppError::ValidationError("Invalid token status".to_string()))?;

        Ok(Token {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            symbol: row.get("symbol"),
            description: row.get("description"),
            token_type,
            total_supply: row.get("total_supply"),
            circulating_supply: row.get("circulating_supply"),
            decimals: row.get("decimals"),
            owner_id: row.get("owner_id"),
            metadata: row.get("metadata"),
            is_active: row.get("is_active"),
            current_price: row.get("current_price"),
            initial_price: row.get("initial_price"),
            contract_address: row.get("contract_address"),
            status,
            metadata_uri: row.get("metadata_uri"),
            compliance_rules: row.get("compliance_rules"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }).transpose()?)
}

pub async fn get_tokens(db: &PgPool, limit: i64, offset: i64) -> Result<Vec<Token>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, project_id, name, symbol, description, token_type, 
            total_supply, circulating_supply, decimals, owner_id, 
            metadata, is_active, current_price, initial_price, 
            contract_address, status, metadata_uri, compliance_rules, 
            created_at, updated_at
        FROM tokens 
        ORDER BY created_at DESC 
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut tokens = Vec::new();
    for row in rows {
        let token_type = row.get::<String, _>("token_type")
            .parse()
            .map_err(|_| AppError::ValidationError("Invalid token type".to_string()))?;
        
        let status = row.get::<String, _>("status")
            .parse()
            .map_err(|_| AppError::ValidationError("Invalid token status".to_string()))?;

        tokens.push(Token {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            symbol: row.get("symbol"),
            description: row.get("description"),
            token_type,
            total_supply: row.get("total_supply"),
            circulating_supply: row.get("circulating_supply"),
            decimals: row.get("decimals"),
            owner_id: row.get("owner_id"),
            metadata: row.get("metadata"),
            is_active: row.get("is_active"),
            current_price: row.get("current_price"),
            initial_price: row.get("initial_price"),
            contract_address: row.get("contract_address"),
            status,
            metadata_uri: row.get("metadata_uri"),
            compliance_rules: row.get("compliance_rules"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }

    Ok(tokens)
}