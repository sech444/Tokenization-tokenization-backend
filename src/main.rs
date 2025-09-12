

//tokenization-backend/src/main.rs

use axum::{
    routing::{put, get, post, delete},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod config;
mod database;
mod handlers;
mod middleware;
mod models;
mod services;
mod utils;

use config::Config;
use database::Database;
use sqlx::Postgres;
use sqlx::Pool;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env()?;

    // Initialize database
    let database = Database::new(&config.database.url).await?;
    database.migrate().await?;

    // Initialize nonce store for wallet authentication
    let nonce_store = handlers::wallet::create_nonce_store();

    // Build our application with routes
    let app = Router::new()
       
        .route("/health", get(health_check))
        
        // Auth routes
        .route("/api/auth/register", post(handlers::auth::register))           // Fixed
        .route("/api/auth/login", post(handlers::auth::login))                 // Fixed
        .route("/api/auth/verify", post(handlers::auth::verify_token))         // Fixed
        .route("/api/auth/logout", post(handlers::auth::logout))               // Added
        .route("/api/auth/change-password", post(handlers::auth::change_password)) // Added
        
        
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
        .route("/api/admin/users", get(handlers::user::list_users))            // Fixed
        .route("/api/admin/kyc/pending", get(handlers::user::pending_kyc))     // Fixed
        .route("/api/admin/kyc/approve", post(handlers::user::approve_kyc))    // Fixed
        
        // Token routes (if handlers exist)
        .route("/api/tokens", get(handlers::token::list_tokens))
        .route("/api/tokens", post(handlers::token::create_token))
        .route("/api/tokens/:id", get(handlers::token::get_token))
        .route("/api/tokens/:id/mint", post(handlers::token::mint_tokens))
        .route("/api/tokens/:id/burn", post(handlers::token::burn_tokens))
        
        // Project routes (if handlers exist)
        .route("/api/projects", get(handlers::project::list_projects))
        .route("/api/projects", post(handlers::project::create_project))
        .route("/api/projects/:id", get(handlers::project::get_project))
        .route("/api/projects/:id/tokenize", post(handlers::project::tokenize_project))


        .route("/api/marketplace/buy", post(handlers::marketplace::buy_tokens))        // POST for buying (action)
        .route("/api/marketplace/:id/orders", get(handlers::marketplace::get_orders))  // GET for retrieving orders
        .route("/api/marketplace/listings", post(handlers::marketplace::create_listing)) // POST for creating
        .route("/api/marketplace/listings", get(handlers::marketplace::get_listings))   // GET for retrieving
        
            
        // Add middleware layers
        .layer(axum::Extension(nonce_store))
        .layer(CorsLayer::permissive())
        .with_state(AppState {
            db: Pool::<Postgres>::connect(&config.database.url).await?,
            config: config.clone(),
        });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Tokenization Platform API is running"
}

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub config: Config,
}

