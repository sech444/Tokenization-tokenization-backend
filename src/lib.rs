// src/lib.rs

// ===== FIX #4: ADD THE NECESSARY IMPORTS =====
use ethers::prelude::{Http, Provider};
use std::sync::Arc;
use sqlx::PgPool;

// Re-export modules to make them accessible throughout the crate
pub mod config;
pub mod contracts;
pub mod database;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;
pub mod utils;

use crate::config::Config;
use crate::services::blockchain::BlockchainService;

// Define a clear type alias for the provider
pub type EthersProvider = Provider<Http>;

// This is the shared state for your entire application
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub blockchain_service: Arc<BlockchainService>,
    // Add the provider field
    pub provider: Arc<EthersProvider>,
}