

// // // //tokenization-backend/src/main.rs


// use axum::{
//     routing::{put, get, post, delete},
//     Router,
// };
// use std::net::SocketAddr;
// use tower_http::cors::CorsLayer;
// use tracing_subscriber;
// use std::sync::Arc;


// use tokenization_platform::config::Config;


// use tokenization_platform::AppState;
// use tokenization_platform::database::Database;
// use tokenization_platform::services::blockchain::{BlockchainService, ContractAddresses};
// use tokenization_platform::handlers; // if you use handlers directly in routes



// // use database::Database;
// use sqlx::{Postgres, Pool};
// use crate::handlers::admin;
// use crate::handlers::auth::list_users;
// // use crate::services::blockchain::BlockchainService;





// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     tracing_subscriber::fmt::init();

//     let config = Config::from_env()?;

//     // Database
//     let database = Database::new(&config.database.url).await?;
//     database.migrate().await?;
//     let pool = database.pool.clone();

//     // Nonce store
//     let nonce_store = handlers::wallet::create_nonce_store();

    
//     // Blockchain service
//     // Convert network to chain_id (you may need to adjust this mapping)
//     let chain_id = match config.blockchain.network.to_lowercase().as_str() {
//         "mainnet" => 1u64,
//         "sepolia" => 11155111u64,
//         "goerli" => 5u64,
//         "polygon" => 137u64,
//         "mumbai" => 80001u64,
//         "amoy"  => 80002u64,
//         _ => return Err(format!("Unsupported network: '{}'. Supported networks: mainnet, sepolia, goerli, polygon, mumbai, amoy", config.blockchain.network).into()),
//     };
    
//     let blockchain_contracts = ContractAddresses {
//         asset_tokenizer: config.blockchain.contract_addresses.token_factory.parse()
//             .map_err(|e| format!("Invalid asset_tokenizer address: {}", e))?,
//         hybrid_tokenizer: "0x4da66DCcEdFde29AE7e7264EE8e1dC40DfdE0129".parse()
//             .map_err(|e| format!("Invalid hybrid_tokenizer address: {}", e))?,
//         compliance_manager: config.blockchain.contract_addresses.compliance.parse()
//             .map_err(|e| format!("Invalid compliance_manager address: {}", e))?,
//         fee_manager: "0x959922bE3CAee4b8Cd9a407cc3ac1C251C2007B1".parse()
//             .map_err(|e| format!("Invalid fee_manager address: {}", e))?,
//         token_registry: "0x9A676e781A523b5d0C0e43731313A708CB607508".parse()
//             .map_err(|e| format!("Invalid token_registry address: {}", e))?,
//         contract_marketplace: config.blockchain.contract_addresses.marketplace.parse()
//             .map_err(|e| format!("Invalid marketplace address: {}", e))?,
//     };

    
//     let blockchain_service = Arc::new(BlockchainService::new(
//         &config.blockchain.rpc_url,
//         &config.blockchain.private_key,
//         chain_id,
//         blockchain_contracts,
//     ).await?);


//     // Router
//     let app = Router::new()
//         .route("/health", get(health_check))

//         // Auth routes
//         .route("/api/auth/register", post(handlers::auth::register))           
//         .route("/api/auth/login", post(handlers::auth::login))                 
//         .route("/api/auth/verify", post(handlers::auth::verify_token))         
//         .route("/api/auth/logout", post(handlers::auth::logout))               
//         .route("/api/auth/change-password", post(handlers::auth::change_password)) 
            
//         // Wallet authentication routes (SIWE - Sign-In With Ethereum)
//         .route("/api/auth/wallet/nonce", post(handlers::wallet::get_wallet_nonce))
//         .route("/api/auth/wallet/verify", post(handlers::wallet::verify_wallet_signature))
//         .route("/api/auth/wallet/disconnect", post(handlers::wallet::disconnect_wallet))
//         .route("/api/auth/wallet/link", post(handlers::wallet::link_wallet_to_user))
//         .route("/api/auth/wallet/info", get(handlers::wallet::get_wallet_info))
            
//         // User routes
//         .route("/api/users/me", get(handlers::user::get_current_user_profile))
//         .route("/api/users/me", put(handlers::user::update_user_profile_current))
//         .route("/api/users/:id", get(handlers::user::get_user_profile))
//         .route("/api/users/:id", put(handlers::user::update_user_profile))
//         .route("/api/users/:id", delete(handlers::user::delete_user_profile))
            
//         // Admin routes
//         .route("/api/admin/users", get(list_users))          
//         .route("/api/admin/kyc/pending", get(admin::pending_kyc))
//         .route("/api/admin/kyc/approve", post(admin::approve_kyc))    
            
//         // Token routes
//         .route("/api/tokens", get(handlers::token::list_tokens))
//         .route("/api/tokens", post(handlers::token::create_token))
//         .route("/api/tokens/:id", get(handlers::token::get_token))
//         .route("/api/tokens/:id/mint", post(handlers::token::mint_tokens))
//         .route("/api/tokens/:id/burn", post(handlers::token::burn_tokens))
            
//         // Project routes
//         .route("/api/projects", get(handlers::project::list_projects))
//         .route("/api/projects", post(handlers::project::create_project))
//         .route("/api/projects/:id", get(handlers::project::get_project))
//         .route("/api/projects/:id/tokenize", post(handlers::project::tokenize_project))

//         // Tokenization & KYC routes
//         .route("/api/projects/:id/tokenization-status", get(handlers::tokenization::get_tokenization_status))
//         .route("/api/kyc/verify", post(handlers::tokenization::verify_kyc))
//         .route("/api/kyc/status/:address", get(handlers::tokenization::check_kyc_status))

//         // Marketplace routes
//         .route("/api/marketplace/buy", post(handlers::marketplace::buy_tokens))        
//         .route("/api/marketplace/:id/orders", get(handlers::marketplace::get_orders))  
//         .route("/api/marketplace/listings", post(handlers::marketplace::create_listing))
//         .route("/api/marketplace/listings", get(handlers::marketplace::get_listings))
        
//         // // Add middleware layers
//         // .layer(axum::Extension(nonce_store))
//         // .layer(CorsLayer::permissive())
//         // .with_state(app_state);

//                     // Add middleware layers
//         .layer(axum::Extension(nonce_store))
//         .layer(CorsLayer::permissive())
//         .with_state(AppState  {
//             db: Pool::<Postgres>::connect(&config.database.url).await?,
//             config: config.clone(),
//             blockchain_service: blockchain_service.clone(),
//         });

//     let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
//     tracing::info!("Server listening on {}", addr);

//     let listener = tokio::net::TcpListener::bind(addr).await?;
//     axum::serve(listener, app).await?;

//     Ok(())
// }

// async fn health_check() -> &'static str {
//     "Tokenization Platform API is running"
// }



use axum::{
    routing::{put, get, post, delete},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;
use std::sync::Arc;

use tokenization_platform::config::Config;
use tokenization_platform::AppState;
use tokenization_platform::database::Database;
use tokenization_platform::services::blockchain::{BlockchainService, ContractAddresses};
use tokenization_platform::handlers;

use sqlx::{Postgres, Pool};
use crate::handlers::admin;
use crate::handlers::auth::list_users;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;

    // Database
    let database = Database::new(&config.database.url).await?;
    database.migrate().await?;
    let pool = database.pool.clone();

    // Nonce store
    let nonce_store = handlers::wallet::create_nonce_store();

    // Blockchain service
    let chain_id = match config.blockchain.network.to_lowercase().as_str() {
        "mainnet" => 1u64,
        "sepolia" => 11155111u64,
        "goerli" => 5u64,
        "polygon" => 137u64,
        "mumbai" => 80001u64,
        "amoy"  => 80002u64,
        _ => return Err(format!("Unsupported network: '{}'. Supported networks: mainnet, sepolia, goerli, polygon, mumbai, amoy", config.blockchain.network).into()),
    };
    
    let blockchain_contracts = ContractAddresses {
        asset_tokenizer: config.blockchain.contract_addresses.token_factory.parse()
            .map_err(|e| format!("Invalid asset_tokenizer address: {}", e))?,
        hybrid_tokenizer: "0x4da66DCcEdFde29AE7e7264EE8e1dC40DfdE0129".parse()
            .map_err(|e| format!("Invalid hybrid_tokenizer address: {}", e))?,
        compliance_manager: config.blockchain.contract_addresses.compliance.parse()
            .map_err(|e| format!("Invalid compliance_manager address: {}", e))?,
        fee_manager: "0x959922bE3CAee4b8Cd9a407cc3ac1C251C2007B1".parse()
            .map_err(|e| format!("Invalid fee_manager address: {}", e))?,
        token_registry: "0x9A676e781A523b5d0C0e43731313A708CB607508".parse()
            .map_err(|e| format!("Invalid token_registry address: {}", e))?,
        contract_marketplace: config.blockchain.contract_addresses.marketplace.parse()
            .map_err(|e| format!("Invalid marketplace address: {}", e))?,
    };

    let blockchain_service = Arc::new(BlockchainService::new(
        &config.blockchain.rpc_url,
        &config.blockchain.private_key,
        chain_id,
        blockchain_contracts,
    ).await?);

    // Router
    let app = Router::new()
        .route("/health", get(health_check))

        // Auth routes
        .route("/api/auth/register", post(handlers::auth::register))           
        .route("/api/auth/login", post(handlers::auth::login))                 
        .route("/api/auth/verify", post(handlers::auth::verify_token))         
        .route("/api/auth/logout", post(handlers::auth::logout))               
        .route("/api/auth/change-password", post(handlers::auth::change_password)) 
            
        // Wallet authentication routes (SIWE - Sign-In With Ethereum)
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
        
        .layer(axum::Extension(nonce_store))
        .layer(CorsLayer::permissive())
        .with_state(AppState  {
            db: Pool::<Postgres>::connect(&config.database.url).await?,
            config: config.clone(),
            blockchain_service: blockchain_service.clone(),
        });

    // 🔥 THE FIX: Use PORT env var first, then fallback to config
    let port = std::env::var("PORT")
        .map(|p| p.parse::<u16>())
        .unwrap_or_else(|_| {
            tracing::info!("PORT env var not found, using config.server.port: {}", config.server.port);
            Ok(config.server.port)
        })?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("🚀 Server starting on {} (port from {})", 
                   addr, 
                   if std::env::var("PORT").is_ok() { "PORT env var" } else { "config" });

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("✅ Server successfully bound and listening on {}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Tokenization Platform API is running"
}