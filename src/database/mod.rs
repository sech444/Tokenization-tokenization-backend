// // tokenization-backend/src/database/mod.rs
// use sqlx::{migrate::Migrator, PgPool};
// use std::error::Error;

// pub mod queries;
// pub mod projects;
// pub mod tokens;
// pub mod transactions;
// pub mod users;
// pub mod wallet_queries;

// static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

// #[derive(Clone)]
// pub struct Database {
//     pub pool: PgPool,
// }

// impl Database {
//     /// Create a new database connection
//     pub async fn new(database_url: &str) -> Result<Self, Box<dyn Error>> {
//         let pool = PgPool::connect(database_url).await?;
//         Ok(Self { pool })
//     }

//     /// Run pending migrations
//     pub async fn migrate(&self) -> Result<(), Box<dyn Error>> {
//         MIGRATOR.run(&self.pool).await?;
//         Ok(())
//     }
// }


// tokenization-backend/src/database/mod.rs
use sqlx::{migrate::Migrator, PgPool};
use sqlx::postgres::PgPoolOptions;
use std::{error::Error, time::Duration};
use tracing::{info, warn};

pub mod queries;
pub mod projects;
pub mod tokens;
pub mod transactions;
pub mod users;
pub mod wallet_queries;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// Create a new database connection with proper pool configuration
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn Error>> {
        info!("Configuring database connection pool...");
        
        let pool = PgPoolOptions::new()
            .max_connections(30)                           // Increase from default 10
            .min_connections(5)                            // Keep minimum connections
            .acquire_timeout(Duration::from_secs(30))      // Wait up to 30s for connection
            .idle_timeout(Duration::from_secs(600))        // Close idle connections after 10 min
            .max_lifetime(Duration::from_secs(1800))       // Recreate connections after 30 min
            .test_before_acquire(true)                     // Validate connections before use
            .connect(database_url)
            .await
            .map_err(|e| {
                warn!("Failed to connect to database: {}", e);
                e
            })?;

        info!(
            "Database pool configured successfully - max_connections: 30, min_connections: 5"
        );

        Ok(Self { pool })
    }

    /// Run pending migrations
    pub async fn migrate(&self) -> Result<(), Box<dyn Error>> {
        info!("Running database migrations...");
        MIGRATOR.run(&self.pool).await?;
        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Get pool health information for monitoring
    pub fn pool_status(&self) -> PoolStatus {
        PoolStatus {
            size: self.pool.size(),
            idle: self.pool.num_idle() as u32,  // Cast usize to u32
            max_connections: 30, // Should match the configuration above
        }
    }

    /// Log pool status for debugging
    pub fn log_pool_status(&self) {
        let status = self.pool_status();
        info!(
            "Pool status - total: {}/{}, idle: {}", 
            status.size, 
            status.max_connections, 
            status.idle
        );
    }
}

#[derive(Debug)]
pub struct PoolStatus {
    pub size: u32,
    pub idle: u32,
    pub max_connections: u32,
}

// Optional: Add a health check method
impl Database {
    /// Perform a simple health check
    pub async fn health_check(&self) -> Result<(), Box<dyn Error>> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}