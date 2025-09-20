// tokenization-backend/src/database/transactions.rs

use crate::models::Transaction;
use crate::utils::errors::AppError;
use sqlx::{PgPool, Row};

pub async fn create_transaction(db: &PgPool, tx: &Transaction) -> Result<Transaction, AppError> {
    let row = sqlx::query(
        r#"
        INSERT INTO transactions (
            user_id, project_id, token_id, transaction_type, amount, fee,
            status, payment_method, description, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING 
            id, user_id, project_id, token_id, transaction_type, amount, fee, status,
            payment_method, payment_reference, blockchain_tx_hash, blockchain_confirmations,
            description, metadata, processed_at, created_at, updated_at
        "#,
    )
    .bind(tx.user_id)
    .bind(tx.project_id)
    .bind(tx.token_id)
    .bind(tx.transaction_type.to_string())
    .bind(tx.amount)
    .bind(tx.fee.unwrap_or(0i64)) // Changed from Decimal::ZERO to 0i64
    .bind(tx.status.to_string())
    .bind(tx.payment_method.as_ref())
    .bind(tx.description.as_ref())
    .bind(&tx.metadata) // Remove as_ref() for JsonValue
    .fetch_one(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Transaction {
        id: row.get("id"),
        user_id: row.get("user_id"),
        project_id: row.get("project_id"),
        token_id: row.get("token_id"),
        transaction_type: row
            .get::<String, _>("transaction_type")
            .parse()
            .map_err(|_| AppError::ValidationError("Invalid transaction type".to_string()))?,
        amount: row.get("amount"),
        fee: row.get("fee"),
        status: row
            .get::<String, _>("status")
            .parse()
            .map_err(|_| AppError::ValidationError("Invalid status".to_string()))?,
        payment_method: row.get("payment_method"),
        payment_reference: row.get("payment_reference"),
        blockchain_tx_hash: row.get("blockchain_tx_hash"),
        blockchain_confirmations: row.get("blockchain_confirmations"),
        description: row.get("description"),
        metadata: row.get("metadata"),
        processed_at: row.get("processed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn get_transaction_by_hash(
    db: &PgPool,
    hash: &str,
) -> Result<Option<Transaction>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            id, user_id, project_id, token_id, transaction_type, amount, fee, status,
            payment_method, payment_reference, blockchain_tx_hash, blockchain_confirmations,
            description, metadata, processed_at, created_at, updated_at
        FROM transactions 
        WHERE blockchain_tx_hash = $1
        "#,
    )
    .bind(hash)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(row
        .map(|r| -> Result<Transaction, AppError> {
            let transaction_type = r
                .get::<String, _>("transaction_type")
                .parse()
                .map_err(|_| AppError::ValidationError("Invalid transaction type".to_string()))?;

            let status = r
                .get::<String, _>("status")
                .parse()
                .map_err(|_| AppError::ValidationError("Invalid status".to_string()))?;

            Ok(Transaction {
                id: r.get("id"),
                user_id: r.get("user_id"),
                project_id: r.get("project_id"),
                token_id: r.get("token_id"),
                transaction_type,
                amount: r.get("amount"),
                fee: r.get("fee"),
                status,
                payment_method: r.get("payment_method"),
                payment_reference: r.get("payment_reference"),
                blockchain_tx_hash: r.get("blockchain_tx_hash"),
                blockchain_confirmations: r.get("blockchain_confirmations"),
                description: r.get("description"),
                metadata: r.get("metadata"),
                processed_at: r.get("processed_at"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
        })
        .transpose()?)
}

/// Count tokens
pub async fn count_tokens(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tokens")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Count transactions
pub async fn count_transactions(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM transactions")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Sum completed transaction volume
pub async fn completed_volume(pool: &PgPool) -> Result<f64, sqlx::Error> {
    let row: (Option<f64>,) =
        sqlx::query_as("SELECT SUM(amount) FROM transactions WHERE status = 'completed'")
            .fetch_one(pool)
            .await?;
    Ok(row.0.unwrap_or(0.0))
}
