


// src/main.rs

use axum::{
    routing::{get, post, put, delete}, // ===== FIX #1: ADD 'put' and 'delete' =====
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokenization_platform::{
    config::Config,
    database::Database,
    handlers,
    services::blockchain::{BlockchainService, ContractAddresses},
    AppState, EthersProvider,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber;

use ethers::prelude::*;
// use sqlx::{Pool, Postgres};

use crate::handlers::admin;
use crate::handlers::auth::list_users;
// use crate::handlers::password_reset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    // config.validate()?; // This method was removed from config.rs, so we remove the call.

    // Database
    let database = Database::new(&config.database.url).await?;
    database.migrate().await?;
    let db_pool = database.pool.clone();

    // Nonce store
    let nonce_store = handlers::wallet::create_nonce_store();

    // Create the ethers provider
    let provider = Provider::<Http>::try_from(&config.blockchain.rpc_url)?;
    let provider = Arc::new(provider);
    
    // Determine the chain ID from the network name in the config
    let chain_id = match config.blockchain.network.to_lowercase().as_str() {
        "mainnet" => 1u64,
        "sepolia" => 11155111u64,
        "goerli" => 5u64,
        "polygon" => 137u64,
        "mumbai" => 80001u64,
        "amoy"  => 80002u64,
        _ => return Err(format!("Unsupported network: '{}'", config.blockchain.network).into()),
    };
    
    // ===== FIX #2: USE THE CORRECT, FLATTENED CONFIG FIELDS =====
    let blockchain_contracts = ContractAddresses {
        asset_tokenizer: config.blockchain.hybrid_asset_tokenizer_proxy_address,
        hybrid_tokenizer: config.blockchain.hybrid_asset_tokenizer_proxy_address,
        compliance_manager: config.blockchain.compliance_manager_proxy_address,
        fee_manager: "0x959922bE3CAee4b8Cd9a407cc3ac1C251C2007B1".parse()?,
        token_registry: "0x9A676e781A523b5d0C0e43731313A708CB607508".parse()?,
        contract_marketplace: config.blockchain.marketplace_core_proxy_address,
    };

    // ===== FIX #3: USE THE CORRECT PRIVATE KEY FIELD NAME =====
    let blockchain_service = Arc::new(BlockchainService::new(
        &config.blockchain.rpc_url,
        &config.blockchain.deployer_private_key, // Use the renamed field
        chain_id,
        blockchain_contracts,
    ).await?);

    // ===== FIX #4: ADD THE PROVIDER TO THE APPSTATE INITIALIZATION =====
    let app_state = AppState  {
        db: db_pool,
        config: config.clone(),
        blockchain_service,
        provider, // Add the provider here
    };

    // Router
    let app = Router::new()
        .route("/health", get(health_check))
        // Auth routes
        .route("/api/auth/register", post(handlers::auth::register))           
        .route("/api/auth/login", post(handlers::auth::login))                 
        .route("/api/auth/verify", post(handlers::auth::verify_token))         
        .route("/api/auth/logout", post(handlers::auth::logout))               
        .route("/api/auth/change-password", post(handlers::auth::change_password)) 
        // Password reset routes
        .route("/api/auth/forgot-password", post(handlers::password_reset::forgot_password))
        .route("/api/auth/reset-password", post(handlers::password_reset::reset_password))
        .route("/api/auth/validate-reset-token", post(handlers::password_reset::validate_reset_token))
        // 🔥 EMAIL VERIFICATION ROUTES - THESE WERE MISSING!
        .route("/api/auth/verify-email", post(handlers::email_verification::verify_email))
        .route("/api/auth/resend-verification", post(handlers::email_verification::resend_verification_email))
        // Wallet authentication routes
        .route("/api/auth/wallet/nonce", post(handlers::wallet::get_wallet_nonce))
        .route("/api/auth/wallet/verify", post(handlers::wallet::verify_wallet_signature))
        .route("/api/auth/wallet/disconnect", post(handlers::wallet::disconnect_wallet))
        .route("/api/auth/wallet/link", post(handlers::wallet::link_wallet_to_user))
        .route("/api/auth/wallet/info", get(handlers::wallet::get_wallet_info))
        // User routes
        .route("/api/users/me", get(handlers::user::get_current_user_profile))
        .route("/api/users/me", put(handlers::user::update_user_profile_current))
        .route("/api/users/:id", get(handlers::user::get_user_profile))
        .route("/api/users/:id", put(handlers::user::update_user_profile))
        .route("/api/users/:id", delete(handlers::user::delete_user_profile))
        // Admin routes
        .route("/api/admin/users", get(list_users))          
        .route("/api/admin/kyc/pending", get(admin::pending_kyc))
        .route("/api/admin/kyc/approve", post(admin::approve_kyc))    
        // Token routes
        .route("/api/tokens", get(handlers::token::list_tokens))
        .route("/api/tokens", post(handlers::token::create_token))
        .route("/api/tokens/:id", get(handlers::token::get_token))
        .route("/api/tokens/:id/mint", post(handlers::token::mint_tokens))
        .route("/api/tokens/:id/burn", post(handlers::token::burn_tokens))
        // Project routes
        .route("/api/projects", get(handlers::project::list_projects))
        .route("/api/projects", post(handlers::project::create_project))
        .route("/api/projects/:id", get(handlers::project::get_project))
        .route("/api/projects/:id/tokenize", post(handlers::project::tokenize_project))
        // Tokenization & KYC routes
        .route("/api/projects/:id/tokenization-status", get(handlers::tokenization::get_tokenization_status))
        .route("/api/kyc/verify", post(handlers::tokenization::verify_kyc))
        .route("/api/kyc/status/:address", get(handlers::tokenization::check_kyc_status))
        // Marketplace routes
        .route("/api/marketplace/buy", post(handlers::marketplace::buy_tokens))        
        .route("/api/marketplace/:id/orders", get(handlers::marketplace::get_orders))  
        .route("/api/marketplace/listings", post(handlers::marketplace::create_listing))
        .route("/api/marketplace/listings", get(handlers::marketplace::get_listings))
        // Add middleware layers
        .layer(axum::Extension(nonce_store))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("🚀 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Tokenization Platform API is running"
}