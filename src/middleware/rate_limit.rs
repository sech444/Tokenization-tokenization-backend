use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use tokio::time::{interval, Interval};
use uuid::Uuid;

use crate::utils::errors::{AppError, AppResult};

// Rate limiting algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitAlgorithm {
    TokenBucket,
    SlidingWindow,
    FixedWindow,
    LeakyBucket,
}

// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub algorithm: RateLimitAlgorithm,
    pub requests_per_window: u32,
    pub window_duration_seconds: u64,
    pub burst_limit: Option<u32>,
    pub storage_backend: StorageBackend,
    pub key_extractor: KeyExtractor,
    pub exempt_user_roles: Vec<String>,
    pub custom_headers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    InMemory,
    Redis { url: String },
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyExtractor {
    IpAddress,
    UserId,
    ApiKey,
    Custom(String),
}

// Rate limit state
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct RateLimitState {
//     pub tokens: u32,
//     pub last_refill: Instant,
//     pub request_timestamps: Vec<Instant>,
// }

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            tokens: 0,
            last_refill: Instant::now(),
            request_timestamps: Vec::new(),
        }
    }
}

// Rate limiter trait
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check_rate_limit(&self, key: &str) -> Result<RateLimitResult, RateLimitError>;
    async fn reset_rate_limit(&self, key: &str) -> Result<(), RateLimitError>;
    async fn get_rate_limit_info(&self, key: &str) -> Result<RateLimitInfo, RateLimitError>;
}

#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub requests_remaining: u32,
    pub reset_time: Option<Instant>,
    pub retry_after_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub current_requests: u32,
    pub limit: u32,
    pub window_start: Instant,
    pub window_end: Instant,
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Key extraction failed: {0}")]
    KeyExtractionError(String),
}

// Main rate limiter service
pub struct RateLimitService {
    config: RateLimitConfig,
    storage: Box<dyn RateLimitStorage + Send + Sync>,
    cleaner: Option<RateLimitCleaner>,
}

impl RateLimitService {
    pub fn new(config: RateLimitConfig) -> Self {
        let storage = Self::create_storage(&config.storage_backend);
        let cleaner = if matches!(config.storage_backend, StorageBackend::InMemory) {
            Some(RateLimitCleaner::new(Arc::clone(&storage)))
        } else {
            None
        };

        Self {
            config,
            storage,
            cleaner,
        }
    }

    fn create_storage(backend: &StorageBackend) -> Box<dyn RateLimitStorage + Send + Sync> {
        match backend {
            StorageBackend::InMemory => Box::new(InMemoryStorage::new()),
            StorageBackend::Redis { url } => Box::new(RedisStorage::new(url)),
            StorageBackend::Database => Box::new(DatabaseStorage::new()),
        }
    }

    fn extract_key(
        &self,
        headers: &HeaderMap,
        user_id: Option<Uuid>,
    ) -> Result<String, RateLimitError> {
        match &self.config.key_extractor {
            KeyExtractor::IpAddress => {
                let ip = headers
                    .get("x-forwarded-for")
                    .or_else(|| headers.get("x-real-ip"))
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("unknown");
                Ok(format!("ip:{}", ip))
            }
            KeyExtractor::UserId => {
                if let Some(id) = user_id {
                    Ok(format!("user:{}", id))
                } else {
                    Err(RateLimitError::KeyExtractionError(
                        "User ID not available".to_string(),
                    ))
                }
            }
            KeyExtractor::ApiKey => {
                let api_key = headers
                    .get("x-api-key")
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| {
                        RateLimitError::KeyExtractionError("API key not found".to_string())
                    })?;
                Ok(format!("api:{}", api_key))
            }
            KeyExtractor::Custom(header_name) => {
                let value = headers
                    .get(header_name)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| {
                        RateLimitError::KeyExtractionError(format!(
                            "Custom header {} not found",
                            header_name
                        ))
                    })?;
                Ok(format!("custom:{}", value))
            }
        }
    }
}

#[async_trait::async_trait]
impl RateLimiter for RateLimitService {
    async fn check_rate_limit(&self, key: &str) -> Result<RateLimitResult, RateLimitError> {
        match self.config.algorithm {
            RateLimitAlgorithm::TokenBucket => self.check_token_bucket(key).await,
            RateLimitAlgorithm::SlidingWindow => self.check_sliding_window(key).await,
            RateLimitAlgorithm::FixedWindow => self.check_fixed_window(key).await,
            RateLimitAlgorithm::LeakyBucket => self.check_leaky_bucket(key).await,
        }
    }

    async fn reset_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        self.storage.delete_state(key).await
    }

    async fn get_rate_limit_info(&self, key: &str) -> Result<RateLimitInfo, RateLimitError> {
        let state = self.storage.get_state(key).await?;
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_duration_seconds);

        Ok(RateLimitInfo {
            current_requests: self.config.requests_per_window - state.tokens,
            limit: self.config.requests_per_window,
            window_start: state.last_refill,
            window_end: state.last_refill + window_duration,
        })
    }
}

impl RateLimitService {
    async fn check_token_bucket(&self, key: &str) -> Result<RateLimitResult, RateLimitError> {
        let mut state = self.storage.get_state(key).await?;
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_duration_seconds);

        // Initialize if first time
        if state.tokens == 0 && state.last_refill.elapsed() > Duration::from_secs(3600) {
            state.tokens = self.config.requests_per_window;
            state.last_refill = now;
        }

        // Refill tokens
        let elapsed = now.duration_since(state.last_refill);
        if elapsed >= window_duration {
            let periods_passed = elapsed.as_secs() / self.config.window_duration_seconds;
            let tokens_to_add = (periods_passed as u32) * self.config.requests_per_window;
            state.tokens = (state.tokens + tokens_to_add).min(self.config.requests_per_window);
            state.last_refill = now;
        }

        // Check if request is allowed
        if state.tokens > 0 {
            state.tokens -= 1;
            self.storage.set_state(key, &state).await?;

            Ok(RateLimitResult {
                allowed: true,
                requests_remaining: state.tokens,
                reset_time: Some(state.last_refill + window_duration),
                retry_after_seconds: None,
            })
        } else {
            let reset_time = state.last_refill + window_duration;
            let retry_after = reset_time.duration_since(now).as_secs();

            Ok(RateLimitResult {
                allowed: false,
                requests_remaining: 0,
                reset_time: Some(reset_time),
                retry_after_seconds: Some(retry_after),
            })
        }
    }

    async fn check_sliding_window(&self, key: &str) -> Result<RateLimitResult, RateLimitError> {
        let mut state = self.storage.get_state(key).await?;
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_duration_seconds);

        // Remove timestamps outside the window
        state
            .request_timestamps
            .retain(|&timestamp| now.duration_since(timestamp) < window_duration);

        // Check if request is allowed
        if state.request_timestamps.len() < self.config.requests_per_window as usize {
            state.request_timestamps.push(now);
            self.storage.set_state(key, &state).await?;

            Ok(RateLimitResult {
                allowed: true,
                requests_remaining: self.config.requests_per_window
                    - state.request_timestamps.len() as u32,
                reset_time: state
                    .request_timestamps
                    .first()
                    .map(|&first| first + window_duration),
                retry_after_seconds: None,
            })
        } else {
            let oldest_request = state.request_timestamps[0];
            let retry_after = (oldest_request + window_duration)
                .duration_since(now)
                .as_secs();

            Ok(RateLimitResult {
                allowed: false,
                requests_remaining: 0,
                reset_time: Some(oldest_request + window_duration),
                retry_after_seconds: Some(retry_after),
            })
        }
    }

    async fn check_fixed_window(&self, key: &str) -> Result<RateLimitResult, RateLimitError> {
        let mut state = self.storage.get_state(key).await?;
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_duration_seconds);

        // Reset window if expired
        if now.duration_since(state.last_refill) >= window_duration {
            state.tokens = self.config.requests_per_window;
            state.last_refill = now;
        }

        // Check if request is allowed
        if state.tokens > 0 {
            state.tokens -= 1;
            self.storage.set_state(key, &state).await?;

            Ok(RateLimitResult {
                allowed: true,
                requests_remaining: state.tokens,
                reset_time: Some(state.last_refill + window_duration),
                retry_after_seconds: None,
            })
        } else {
            let reset_time = state.last_refill + window_duration;
            let retry_after = reset_time.duration_since(now).as_secs();

            Ok(RateLimitResult {
                allowed: false,
                requests_remaining: 0,
                reset_time: Some(reset_time),
                retry_after_seconds: Some(retry_after),
            })
        }
    }

    async fn check_leaky_bucket(&self, key: &str) -> Result<RateLimitResult, RateLimitError> {
        // Leaky bucket implementation
        // For simplicity, using token bucket with continuous refill
        self.check_token_bucket(key).await
    }
}

// Storage trait
#[async_trait::async_trait]
pub trait RateLimitStorage: Send + Sync {
    async fn get_state(&self, key: &str) -> Result<RateLimitState, RateLimitError>;
    async fn set_state(&self, key: &str, state: &RateLimitState) -> Result<(), RateLimitError>;
    async fn delete_state(&self, key: &str) -> Result<(), RateLimitError>;
}

// In-memory storage implementation
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<String, RateLimitState>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl RateLimitStorage for InMemoryStorage {
    async fn get_state(&self, key: &str) -> Result<RateLimitState, RateLimitError> {
        let data = self.data.read().unwrap();
        Ok(data.get(key).cloned().unwrap_or_default())
    }

    async fn set_state(&self, key: &str, state: &RateLimitState) -> Result<(), RateLimitError> {
        let mut data = self.data.write().unwrap();
        data.insert(key.to_string(), state.clone());
        Ok(())
    }

    async fn delete_state(&self, key: &str) -> Result<(), RateLimitError> {
        let mut data = self.data.write().unwrap();
        data.remove(key);
        Ok(())
    }
}

// Redis storage implementation (placeholder)
pub struct RedisStorage {
    _url: String,
}

impl RedisStorage {
    pub fn new(url: &str) -> Self {
        Self {
            _url: url.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl RateLimitStorage for RedisStorage {
    async fn get_state(&self, _key: &str) -> Result<RateLimitState, RateLimitError> {
        // TODO: Implement Redis storage
        Ok(RateLimitState::default())
    }

    async fn set_state(&self, _key: &str, _state: &RateLimitState) -> Result<(), RateLimitError> {
        // TODO: Implement Redis storage
        Ok(())
    }

    async fn delete_state(&self, _key: &str) -> Result<(), RateLimitError> {
        // TODO: Implement Redis storage
        Ok(())
    }
}

// Database storage implementation (placeholder)
pub struct DatabaseStorage;

impl DatabaseStorage {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl RateLimitStorage for DatabaseStorage {
    async fn get_state(&self, _key: &str) -> Result<RateLimitState, RateLimitError> {
        // TODO: Implement database storage
        Ok(RateLimitState::default())
    }

    async fn set_state(&self, _key: &str, _state: &RateLimitState) -> Result<(), RateLimitError> {
        // TODO: Implement database storage
        Ok(())
    }

    async fn delete_state(&self, _key: &str) -> Result<(), RateLimitError> {
        // TODO: Implement database storage
        Ok(())
    }
}

// Cleanup service for in-memory storage
pub struct RateLimitCleaner {
    storage: Arc<dyn RateLimitStorage + Send + Sync>,
    _interval: Interval,
}

impl RateLimitCleaner {
    pub fn new(storage: Arc<dyn RateLimitStorage + Send + Sync>) -> Self {
        let interval = interval(Duration::from_secs(300)); // Clean every 5 minutes

        Self {
            storage,
            _interval: interval,
        }
    }

    pub async fn start_cleanup_task(&mut self) {
        loop {
            self._interval.tick().await;
            self.cleanup_expired_entries().await;
        }
    }

    async fn cleanup_expired_entries(&self) {
        // TODO: Implement cleanup logic for expired entries
        log::debug!("Running rate limit cleanup task");
    }
}

// Middleware function
pub async fn rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimitService>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract user ID from request if available (this would depend on your auth system)
    let user_id = None; // TODO: Extract from auth context

    // Extract key for rate limiting
    let key = rate_limiter
        .extract_key(&headers, user_id)
        .map_err(|e| AppError::RateLimitError(e.to_string()))?;

    // Check rate limit
    let result = rate_limiter
        .check_rate_limit(&key)
        .await
        .map_err(|e| AppError::RateLimitError(e.to_string()))?;

    if !result.allowed {
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.",
        )
            .into_response();

        // Add rate limit headers
        if rate_limiter.config.custom_headers {
            let headers = response.headers_mut();
            headers.insert(
                "X-RateLimit-Limit",
                rate_limiter.config.requests_per_window.into(),
            );
            headers.insert("X-RateLimit-Remaining", result.requests_remaining.into());

            if let Some(retry_after) = result.retry_after_seconds {
                headers.insert("Retry-After", retry_after.into());
            }

            if let Some(reset_time) = result.reset_time {
                let reset_timestamp = reset_time
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                headers.insert("X-RateLimit-Reset", reset_timestamp.into());
            }
        }

        return Ok(response);
    }

    // Continue with request
    let mut response = next.run(request).await;

    // Add rate limit headers to successful responses
    if rate_limiter.config.custom_headers {
        let headers = response.headers_mut();
        headers.insert(
            "X-RateLimit-Limit",
            rate_limiter.config.requests_per_window.into(),
        );
        headers.insert("X-RateLimit-Remaining", result.requests_remaining.into());

        if let Some(reset_time) = result.reset_time {
            let reset_timestamp = reset_time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            headers.insert("X-RateLimit-Reset", reset_timestamp.into());
        }
    }

    Ok(response)
}

// Default configurations
impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            algorithm: RateLimitAlgorithm::TokenBucket,
            requests_per_window: 100,
            window_duration_seconds: 3600, // 1 hour
            burst_limit: Some(10),
            storage_backend: StorageBackend::InMemory,
            key_extractor: KeyExtractor::IpAddress,
            exempt_user_roles: vec!["admin".to_string()],
            custom_headers: true,
        }
    }
}

// Helper functions for creating common rate limit configurations
impl RateLimitConfig {
    pub fn per_ip(requests: u32, window_seconds: u64) -> Self {
        Self {
            requests_per_window: requests,
            window_duration_seconds: window_seconds,
            key_extractor: KeyExtractor::IpAddress,
            ..Default::default()
        }
    }

    pub fn per_user(requests: u32, window_seconds: u64) -> Self {
        Self {
            requests_per_window: requests,
            window_duration_seconds: window_seconds,
            key_extractor: KeyExtractor::UserId,
            ..Default::default()
        }
    }

    pub fn per_api_key(requests: u32, window_seconds: u64) -> Self {
        Self {
            requests_per_window: requests,
            window_duration_seconds: window_seconds,
            key_extractor: KeyExtractor::ApiKey,
            ..Default::default()
        }
    }

    pub fn with_algorithm(mut self, algorithm: RateLimitAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    pub fn with_storage(mut self, storage: StorageBackend) -> Self {
        self.storage_backend = storage;
        self
    }
}
