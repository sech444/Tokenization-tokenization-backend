// // src/models/transaction.rs

// use chrono::{DateTime, Utc};
// use serde::{Deserialize, Serialize};
// use sqlx::FromRow;
// use uuid::Uuid;
// use validator::Validate;

// #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
// pub struct Transaction {
//     pub id: Uuid,
//     pub user_id: Uuid,
//     pub project_id: Option<Uuid>,
//     pub token_id: Option<Uuid>,
//     pub transaction_type: TransactionType,
//     pub amount: i64,      // in cents
//     pub fee: Option<i64>, // in cents
//     pub status: TransactionStatus,
//     pub payment_method: Option<String>,
//     pub payment_reference: Option<String>,
//     pub blockchain_tx_hash: Option<String>,
//     pub blockchain_confirmations: Option<i32>,
//     pub description: Option<String>,
//     pub metadata: serde_json::Value,
//     pub processed_at: Option<DateTime<Utc>>,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: DateTime<Utc>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
// #[sqlx(type_name = "transaction_type", rename_all = "lowercase")]
// pub enum TransactionType {
//     Investment,
//     Withdrawal,
//     Transfer,
//     Dividend,
//     Fee,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
// #[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
// pub enum TransactionStatus {
//     Pending,
//     Processing,
//     Completed,
//     Failed,
//     Cancelled,
// }

// #[derive(Debug, Deserialize, Validate)]
// pub struct CreateTransactionRequest {
//     pub project_id: Option<Uuid>,
//     pub token_id: Option<Uuid>,
//     pub transaction_type: TransactionType,

//     #[validate(range(min = 1))]
//     pub amount: i64, // in cents

//     pub payment_method: Option<String>,
//     pub description: Option<String>,
//     pub metadata: Option<serde_json::Value>,
// }

// #[derive(Debug, Deserialize, Validate)]
// pub struct UpdateTransactionRequest {
//     pub status: Option<TransactionStatus>,
//     pub payment_reference: Option<String>,
//     pub blockchain_tx_hash: Option<String>,
//     pub blockchain_confirmations: Option<i32>,
//     pub description: Option<String>,
//     pub metadata: Option<serde_json::Value>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct TransactionResponse {
//     pub id: Uuid,
//     pub user_id: Uuid,
//     pub project_id: Option<Uuid>,
//     pub token_id: Option<Uuid>,
//     pub transaction_type: TransactionType,
//     pub amount: i64,
//     pub fee: Option<i64>,
//     pub status: TransactionStatus,
//     pub payment_method: Option<String>,
//     pub payment_reference: Option<String>,
//     pub blockchain_tx_hash: Option<String>,
//     pub blockchain_confirmations: Option<i32>,
//     pub description: Option<String>,
//     pub processed_at: Option<DateTime<Utc>>,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: DateTime<Utc>,
// }

// impl From<Transaction> for TransactionResponse {
//     fn from(transaction: Transaction) -> Self {
//         Self {
//             id: transaction.id,
//             user_id: transaction.user_id,
//             project_id: transaction.project_id,
//             token_id: transaction.token_id,
//             transaction_type: transaction.transaction_type,
//             amount: transaction.amount,
//             fee: transaction.fee,
//             status: transaction.status,
//             payment_method: transaction.payment_method,
//             payment_reference: transaction.payment_reference,
//             blockchain_tx_hash: transaction.blockchain_tx_hash,
//             blockchain_confirmations: transaction.blockchain_confirmations,
//             description: transaction.description,
//             processed_at: transaction.processed_at,
//             created_at: transaction.created_at,
//             updated_at: transaction.updated_at,
//         }
//     }
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct TransactionSummary {
//     pub total_transactions: i64,
//     pub total_volume: i64,
//     pub completed_transactions: i64,
//     pub pending_transactions: i64,
//     pub failed_transactions: i64,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct UserTransactionHistory {
//     pub user_id: Uuid,
//     pub transactions: Vec<TransactionResponse>,
//     pub total_count: i64,
//     pub total_invested: i64,
//     pub total_withdrawn: i64,
// }


// src/models/transaction.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub token_id: Option<Uuid>,
    pub transaction_type: TransactionType,
    pub amount: i64,      // in cents
    pub fee: Option<i64>, // in cents
    pub status: TransactionStatus,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub blockchain_tx_hash: Option<String>,
    pub blockchain_confirmations: Option<i32>,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type", rename_all = "lowercase")]
pub enum TransactionType {
    Investment,
    Withdrawal,
    Transfer,
    Dividend,
    Fee,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TransactionType::Investment => "investment",
            TransactionType::Withdrawal => "withdrawal",
            TransactionType::Transfer => "transfer",
            TransactionType::Dividend => "dividend",
            TransactionType::Fee => "fee",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for TransactionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "investment" => Ok(TransactionType::Investment),
            "withdrawal" => Ok(TransactionType::Withdrawal),
            "transfer" => Ok(TransactionType::Transfer),
            "dividend" => Ok(TransactionType::Dividend),
            "fee" => Ok(TransactionType::Fee),
            _ => Err(format!("Invalid transaction type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Processing => "processing",
            TransactionStatus::Completed => "completed",
            TransactionStatus::Failed => "failed",
            TransactionStatus::Cancelled => "cancelled",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for TransactionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TransactionStatus::Pending),
            "processing" => Ok(TransactionStatus::Processing),
            "completed" => Ok(TransactionStatus::Completed),
            "failed" => Ok(TransactionStatus::Failed),
            "cancelled" => Ok(TransactionStatus::Cancelled),
            _ => Err(format!("Invalid transaction status: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTransactionRequest {
    pub project_id: Option<Uuid>,
    pub token_id: Option<Uuid>,
    pub transaction_type: TransactionType,

    #[validate(range(min = 1))]
    pub amount: i64, // in cents

    pub payment_method: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTransactionRequest {
    pub status: Option<TransactionStatus>,
    pub payment_reference: Option<String>,
    pub blockchain_tx_hash: Option<String>,
    pub blockchain_confirmations: Option<i32>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub token_id: Option<Uuid>,
    pub transaction_type: TransactionType,
    pub amount: i64,
    pub fee: Option<i64>,
    pub status: TransactionStatus,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub blockchain_tx_hash: Option<String>,
    pub blockchain_confirmations: Option<i32>,
    pub description: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Transaction> for TransactionResponse {
    fn from(transaction: Transaction) -> Self {
        Self {
            id: transaction.id,
            user_id: transaction.user_id,
            project_id: transaction.project_id,
            token_id: transaction.token_id,
            transaction_type: transaction.transaction_type,
            amount: transaction.amount,
            fee: transaction.fee,
            status: transaction.status,
            payment_method: transaction.payment_method,
            payment_reference: transaction.payment_reference,
            blockchain_tx_hash: transaction.blockchain_tx_hash,
            blockchain_confirmations: transaction.blockchain_confirmations,
            description: transaction.description,
            processed_at: transaction.processed_at,
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub total_transactions: i64,
    pub total_volume: i64,
    pub completed_transactions: i64,
    pub pending_transactions: i64,
    pub failed_transactions: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTransactionHistory {
    pub user_id: Uuid,
    pub transactions: Vec<TransactionResponse>,
    pub total_count: i64,
    pub total_invested: i64,
    pub total_withdrawn: i64,
}

