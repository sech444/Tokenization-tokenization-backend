// // src/lib.rs
// use sqlx::PgPool;
// use crate::services::blockchain::BlockchainService;
// use std::sync::Arc;

// pub mod config;
// pub mod database;
// pub mod handlers;
// pub mod middleware;
// pub mod models;
// pub mod services;
// pub mod utils;


// pub use config::Config;
// pub use database::Database;

// // Re-export commonly used types
// pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
// pub blockchain_service: Arc<BlockchainService>;


// #[derive(Clone)]
// pub struct AppState {
//     pub db: PgPool,
//     pub config: Config,
//     pub blockchain_service: Arc<BlockchainService>
// }


use sqlx::PgPool;
use std::sync::Arc;

pub mod config;
pub mod database;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;
pub mod utils;

use crate::services::blockchain::BlockchainService;

pub use config::Config;
pub use database::Database;

// Re-export commonly used types
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub blockchain_service: Arc<BlockchainService>,  // ✅ this is where it belongs
}
