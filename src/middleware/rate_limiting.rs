// src/middleware/rate_limiting.rs

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Extension,
};
use chrono::{DateTime, Utc};
use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    models::user::{User, UserRole},
    services::audit::{AuditEventType, AuditService},
    utils::errors::{AppError, AppResult},
    AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_limit: u32,
    pub window_size_seconds: u64,
    pub block_duration_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_limit: 10,
            window_size_seconds: 60,
            block_duration_seconds: 300, // 5 minutes
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitRules {
    pub ip_limits: RateLimitConfig,
    pub user_limits: RateLimitConfig,
    pub endpoint_limits: HashMap<String, RateLimitConfig>,
    pub role_limits: HashMap<UserRole, RateLimitConfig>,
    pub global_limits: RateLimitConfig,
}

impl Default for RateLimitRules {
    fn default() -> Self {
        let mut endpoint_limits = HashMap::new();
        let mut role_limits = HashMap::new();

        // Define endpoint-specific limits
        endpoint_limits.insert(
            "/api/auth/login".to_string(),
            RateLimitConfig {
                requests_per_minute: 5,
                requests_per_hour: 20,
                requests_per_day: 100,
                burst_limit: 3,
                window_size_seconds: 60,
                block_duration_seconds: 900, // 15 minutes for failed logins
            },
        );

        endpoint_limits.insert(
            "/api/auth/register".to_string(),
            RateLimitConfig {
                requests_per_minute: 2,
                requests_per_hour: 10,
                requests_per_day: 20,
                burst_limit: 1,
                window_size_seconds: 60,
                block_duration_seconds: 1800, // 30 minutes
            },
        );

        endpoint_limits.insert(
            "/api/tokens".to_string(),
            RateLimitConfig {
                requests_per_minute: 10,
                requests_per_hour: 100,
                requests_per_day: 500,
                burst_limit: 5,
                window_size_seconds: 60,
                block_duration_seconds: 300,
            },
        );

        endpoint_limits.insert(
            "/api/projects".to_string(),
            RateLimitConfig {
                requests_per_minute: 20,
                requests_per_hour: 200,
                requests_per_day: 1000,
                burst_limit: 10,
                window_size_seconds: 60,
                block_duration_seconds: 300,
            },
        );

        // High-risk operations
        endpoint_limits.insert(
            "/api/admin".to_string(),
            RateLimitConfig {
                requests_per_minute: 30,
                requests_per_hour: 300,
                requests_per_day: 2000,
                burst_limit: 15,
                window_size_seconds: 60,
                block_duration_seconds: 600, // 10 minutes
            },
        );

        // Role-based limits
        role_limits.insert(
            UserRole::Admin,
            RateLimitConfig {
                requests_per_minute: 200,
                requests_per_hour: 5000,
                requests_per_day: 50000,
                burst_limit: 50,
                window_size_seconds: 60,
                block_duration_seconds: 60,
            },
        );

        role_limits.insert(
            UserRole::ProjectManager,
            RateLimitConfig {
                requests_per_minute: 100,
                requests_per_hour: 2000,
                requests_per_day: 20000,
                burst_limit: 25,
                window_size_seconds: 60,
                block_duration_seconds: 180,
            },
        );

        role_limits.insert(
            UserRole::User,
            RateLimitConfig {
                requests_per_minute: 60,
                requests_per_hour: 1000,
                requests_per_day: 10000,
                burst_limit: 10,
                window_size_seconds: 60,
                block_duration_seconds: 300,
            },
        );

        role_limits.insert(
            UserRole::Investor,
            RateLimitConfig {
                requests_per_minute: 80,
                requests_per_hour: 1500,
                requests_per_day: 15000,
                burst_limit: 15,
                window_size_seconds: 60,
                block_duration_seconds: 240,
            },
        );

        Self {
            ip_limits: RateLimitConfig::default(),
            user_limits: RateLimitConfig::default(),
            endpoint_limits,
            role_limits,
            global_limits: RateLimitConfig {
                requests_per_minute: 10000,
                requests_per_hour: 100000,
                requests_per_day: 1000000,
                burst_limit: 1000,
                window_size_seconds: 60,
                block_duration_seconds: 60,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_time: DateTime<Utc>,
    pub retry_after: Option<u64>,
    pub blocked: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub count: u32,
    pub first_request: DateTime<Utc>,
    pub last_request: DateTime<Utc>,
    pub blocked_until: Option<DateTime<Utc>>,
    pub violation_count: u32,
}

pub struct RateLimiter {
    redis_client: RedisClient,
    rules: RateLimitRules,
    audit_service: AuditService,
}

impl RateLimiter {
    pub fn new(
        redis_url: &str,
        rules: RateLimitRules,
        audit_service: AuditService,
    ) -> AppResult<Self> {
        let redis_client = RedisClient::open(redis_url).map_err(|e| {
            AppError::InternalServerError(format!("Failed to connect to Redis: {}", e))
        })?;

        Ok(Self {
            redis_client,
            rules,
            audit_service,
        })
    }

    /// Check rate limit for a request
    pub async fn check_rate_limit(
        &self,
        identifier: &str,
        config: &RateLimitConfig,
        request_type: &str,
    ) -> AppResult<RateLimitInfo> {
        let mut connection = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Redis connection failed: {}", e))
            })?;

        let now = Utc::now();
        let window_key = format!("ratelimit:{}:{}", identifier, request_type);
        let block_key = format!("block:{}", window_key);

        // Check if currently blocked
        let blocked_until: Option<String> = connection.get(&block_key).await.unwrap_or(None);
        if let Some(blocked_until_str) = blocked_until {
            if let Ok(blocked_until_time) = DateTime::parse_from_rfc3339(&blocked_until_str) {
                if now < blocked_until_time.with_timezone(&Utc) {
                    return Ok(RateLimitInfo {
                        limit: config.requests_per_minute,
                        remaining: 0,
                        reset_time: blocked_until_time.with_timezone(&Utc),
                        retry_after: Some(
                            (blocked_until_time.with_timezone(&Utc) - now).num_seconds() as u64,
                        ),
                        blocked: true,
                        reason: Some("Rate limit exceeded - temporarily blocked".to_string()),
                    });
                } else {
                    // Block expired, remove it
                    let _: () = connection.del(&block_key).await.unwrap_or(());
                }
            }
        }

        // Sliding window rate limiting
        let window_start = now - chrono::Duration::seconds(config.window_size_seconds as i64);
        let window_start_timestamp = window_start.timestamp() as f64;
        let now_timestamp = now.timestamp() as f64;

        // Remove old entries
        let _: () = connection
            .zrembyscore(&window_key, 0, window_start_timestamp)
            .await
            .unwrap_or(());

        // Count current requests in window
        let current_count: u32 = connection.zcard(&window_key).await.unwrap_or(0);

        // Check various limits
        let mut violated_limit = None;
        let mut limit_value = config.requests_per_minute;

        if current_count >= config.requests_per_minute {
            violated_limit = Some("requests_per_minute");
            limit_value = config.requests_per_minute;
        }

        // Check hourly limit
        let hour_key = format!("{}:hour", window_key);
        let hour_start = now - chrono::Duration::hours(1);
        let hour_count: u32 = connection.zcard(&hour_key).await.unwrap_or(0);

        if hour_count >= config.requests_per_hour {
            violated_limit = Some("requests_per_hour");
            limit_value = config.requests_per_hour;
        }

        // Check daily limit
        let day_key = format!("{}:day", window_key);
        let day_count: u32 = connection.zcard(&day_key).await.unwrap_or(0);

        if day_count >= config.requests_per_day {
            violated_limit = Some("requests_per_day");
            limit_value = config.requests_per_day;
        }

        if let Some(limit_type) = violated_limit {
            // Rate limit exceeded - implement exponential backoff
            self.handle_rate_limit_violation(
                &mut connection,
                &window_key,
                &block_key,
                config,
                identifier,
                limit_type,
            )
            .await?;

            return Ok(RateLimitInfo {
                limit: limit_value,
                remaining: 0,
                reset_time: now + chrono::Duration::seconds(config.block_duration_seconds as i64),
                retry_after: Some(config.block_duration_seconds),
                blocked: true,
                reason: Some(format!("Rate limit exceeded: {}", limit_type)),
            });
        }

        // Add current request to window
        let request_id = Uuid::new_v4().to_string();
        let _: () = connection
            .zadd(&window_key, request_id, now_timestamp)
            .await
            .unwrap_or(());
        let _: () = connection
            .zadd(&hour_key, request_id.clone(), now_timestamp)
            .await
            .unwrap_or(());
        let _: () = connection
            .zadd(&day_key, request_id, now_timestamp)
            .await
            .unwrap_or(());

        // Set expiration
        let _: () = connection
            .expire(&window_key, config.window_size_seconds as usize)
            .await
            .unwrap_or(());
        let _: () = connection.expire(&hour_key, 3600).await.unwrap_or(());
        let _: () = connection.expire(&day_key, 86400).await.unwrap_or(());

        let remaining = config.requests_per_minute.saturating_sub(current_count + 1);
        let reset_time = now + chrono::Duration::seconds(config.window_size_seconds as i64);

        Ok(RateLimitInfo {
            limit: config.requests_per_minute,
            remaining,
            reset_time,
            retry_after: None,
            blocked: false,
            reason: None,
        })
    }

    /// Handle rate limit violation with escalating penalties
    async fn handle_rate_limit_violation(
        &self,
        connection: &mut redis::aio::Connection,
        window_key: &str,
        block_key: &str,
        config: &RateLimitConfig,
        identifier: &str,
        limit_type: &str,
    ) -> AppResult<()> {
        let violation_key = format!("violations:{}", identifier);
        let violation_count: u32 = connection.incr(&violation_key, 1).await.unwrap_or(1);
        let _: () = connection.expire(&violation_key, 3600).await.unwrap_or(()); // Reset violations after 1 hour

        // Exponential backoff based on violation count
        let block_duration =
            config.block_duration_seconds * (2_u64.pow(violation_count.min(5) - 1));
        let block_until = Utc::now() + chrono::Duration::seconds(block_duration as i64);

        let _: () = connection
            .set(&block_key, block_until.to_rfc3339())
            .await
            .unwrap_or(());
        let _: () = connection
            .expire(block_key, block_duration as usize)
            .await
            .unwrap_or(());

        // Log security event for multiple violations
        if violation_count >= 3 {
            warn!(
                "Suspicious activity: {} violations from {}",
                violation_count, identifier
            );

            self.audit_service
                .log_security_event(
                    AuditEventType::SuspiciousActivity,
                    format!(
                        "Rate limit violations: {} from {}",
                        violation_count, identifier
                    ),
                    None,
                    None,
                )
                .await?;
        }

        info!(
            "Rate limit violation: {} for {} (violation #{}, blocked for {}s)",
            limit_type, identifier, violation_count, block_duration
        );

        Ok(())
    }

    /// Get appropriate rate limit config for request
    fn get_rate_limit_config(&self, endpoint: &str, user: Option<&User>) -> RateLimitConfig {
        // Check endpoint-specific limits first
        for (endpoint_pattern, config) in &self.rules.endpoint_limits {
            if endpoint.starts_with(endpoint_pattern) {
                return config.clone();
            }
        }

        // Check role-based limits
        if let Some(user) = user {
            if let Some(config) = self.rules.role_limits.get(&user.role) {
                return config.clone();
            }
        }

        // Default to user limits
        self.rules.user_limits.clone()
    }

    /// Extract IP address from request
    fn extract_ip_address(headers: &HeaderMap) -> String {
        headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim())
            .or_else(|| headers.get("x-real-ip").and_then(|h| h.to_str().ok()))
            .unwrap_or("unknown")
            .to_string()
    }

    /// Check if IP is in whitelist
    async fn is_whitelisted(&self, ip: &str) -> bool {
        let mut connection = match self.redis_client.get_async_connection().await {
            Ok(conn) => conn,
            Err(_) => return false,
        };

        let whitelist_key = "whitelist:ips";
        let is_whitelisted: bool = connection
            .sismember(&whitelist_key, ip)
            .await
            .unwrap_or(false);

        is_whitelisted
    }

    /// Check for suspicious patterns
    async fn detect_suspicious_activity(
        &self,
        ip: &str,
        user_id: Option<Uuid>,
        endpoint: &str,
    ) -> AppResult<bool> {
        let mut connection = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Redis connection failed: {}", e))
            })?;

        // Check for rapid endpoint switching
        let pattern_key = format!(
            "pattern:{}:{}",
            ip,
            user_id
                .map(|u| u.to_string())
                .unwrap_or_else(|| "anonymous".to_string())
        );
        let _: () = connection.lpush(&pattern_key, endpoint).await.unwrap_or(());
        let _: () = connection.ltrim(&pattern_key, 0, 10).await.unwrap_or(()); // Keep last 10 requests
        let _: () = connection.expire(&pattern_key, 300).await.unwrap_or(()); // 5 minutes

        let recent_endpoints: Vec<String> = connection
            .lrange(&pattern_key, 0, -1)
            .await
            .unwrap_or_default();
        let unique_endpoints: std::collections::HashSet<_> = recent_endpoints.into_iter().collect();

        // Suspicious if hitting many different endpoints rapidly
        if unique_endpoints.len() > 8 {
            return Ok(true);
        }

        // Check for distributed attacks (same user from multiple IPs)
        if let Some(user_id) = user_id {
            let user_ips_key = format!("user_ips:{}", user_id);
            let _: () = connection.sadd(&user_ips_key, ip).await.unwrap_or(());
            let _: () = connection.expire(&user_ips_key, 1800).await.unwrap_or(()); // 30 minutes

            let ip_count: u32 = connection.scard(&user_ips_key).await.unwrap_or(0);
            if ip_count > 5 {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// Main rate limiting middleware
pub async fn rate_limiting_middleware<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let headers = request.headers();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let endpoint = uri.path();

    // Extract request information
    let ip_address = RateLimiter::extract_ip_address(headers);
    let user = request.extensions().get::<User>().cloned();
    let user_id = user.as_ref().map(|u| u.id);

    // Initialize rate limiter (in production, this would be a singleton)
    let audit_service = AuditService::new(state.db.clone(), state.config.jwt.secret.clone());
    let rules = RateLimitRules::default();
    let rate_limiter = RateLimiter::new(&state.config.redis.url, rules, audit_service)?;

    // Skip rate limiting for whitelisted IPs
    if rate_limiter.is_whitelisted(&ip_address).await {
        return Ok(next.run(request).await);
    }

    // Skip rate limiting for admin users (optional)
    if let Some(user) = &user {
        if matches!(user.role, UserRole::Admin) && endpoint.starts_with("/api/admin") {
            return Ok(next.run(request).await);
        }
    }

    // Detect suspicious activity
    let is_suspicious = rate_limiter
        .detect_suspicious_activity(&ip_address, user_id, endpoint)
        .await?;
    if is_suspicious {
        warn!(
            "Suspicious activity detected from {} for endpoint {}",
            ip_address, endpoint
        );
        return Err(AppError::TooManyRequests {
            message: "Suspicious activity detected".to_string(),
            retry_after: Some(3600), // 1 hour penalty
        });
    }

    // Get rate limit configuration
    let config = rate_limiter.get_rate_limit_config(endpoint, user.as_ref());

    // Check IP-based rate limits
    let ip_identifier = format!("ip:{}", ip_address);
    let ip_limit_info = rate_limiter
        .check_rate_limit(&ip_identifier, &config, "ip")
        .await?;

    if ip_limit_info.blocked {
        return Err(AppError::TooManyRequests {
            message: ip_limit_info
                .reason
                .unwrap_or_else(|| "IP rate limit exceeded".to_string()),
            retry_after: ip_limit_info.retry_after,
        });
    }

    // Check user-based rate limits if authenticated
    if let Some(user) = &user {
        let user_identifier = format!("user:{}", user.id);
        let user_limit_info = rate_limiter
            .check_rate_limit(&user_identifier, &config, "user")
            .await?;

        if user_limit_info.blocked {
            return Err(AppError::TooManyRequests {
                message: user_limit_info
                    .reason
                    .unwrap_or_else(|| "User rate limit exceeded".to_string()),
                retry_after: user_limit_info.retry_after,
            });
        }
    }

    // Check endpoint-specific rate limits
    let endpoint_identifier = format!("endpoint:{}:{}", endpoint, ip_address);
    let endpoint_limit_info = rate_limiter
        .check_rate_limit(&endpoint_identifier, &config, "endpoint")
        .await?;

    if endpoint_limit_info.blocked {
        return Err(AppError::TooManyRequests {
            message: endpoint_limit_info
                .reason
                .unwrap_or_else(|| "Endpoint rate limit exceeded".to_string()),
            retry_after: endpoint_limit_info.retry_after,
        });
    }

    // Add rate limit information to request for downstream handlers
    request.extensions_mut().insert(ip_limit_info.clone());

    // Process the request
    let mut response = next.run(request).await;

    // Add rate limit headers to response
    let response_headers = response.headers_mut();
    response_headers.insert(
        "X-RateLimit-Limit",
        ip_limit_info.limit.to_string().parse().unwrap(),
    );
    response_headers.insert(
        "X-RateLimit-Remaining",
        ip_limit_info.remaining.to_string().parse().unwrap(),
    );
    response_headers.insert(
        "X-RateLimit-Reset",
        ip_limit_info
            .reset_time
            .timestamp()
            .to_string()
            .parse()
            .unwrap(),
    );

    if let Some(retry_after) = ip_limit_info.retry_after {
        response_headers.insert("Retry-After", retry_after.to_string().parse().unwrap());
    }

    Ok(response)
}

/// Middleware for high-security endpoints with stricter limits
pub async fn strict_rate_limiting_middleware<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // Apply stricter limits for sensitive operations
    let headers = request.headers();
    let ip_address = RateLimiter::extract_ip_address(headers);

    let audit_service = AuditService::new(state.db.clone(), state.config.jwt.secret.clone());
    let mut rules = RateLimitRules::default();

    // Override with stricter limits
    rules.ip_limits = RateLimitConfig {
        requests_per_minute: 10,
        requests_per_hour: 100,
        requests_per_day: 1000,
        burst_limit: 3,
        window_size_seconds: 60,
        block_duration_seconds: 1800, // 30 minutes
    };

    let rate_limiter = RateLimiter::new(&state.config.redis.url, rules, audit_service)?;

    let ip_identifier = format!("strict:ip:{}", ip_address);
    let limit_info = rate_limiter
        .check_rate_limit(&ip_identifier, &rate_limiter.rules.ip_limits, "strict")
        .await?;

    if limit_info.blocked {
        return Err(AppError::TooManyRequests {
            message: "Strict rate limit exceeded for sensitive operation".to_string(),
            retry_after: limit_info.retry_after,
        });
    }

    rate_limiting_middleware(State(state), request, next).await
}

/// Configuration for different endpoint types
pub mod endpoint_configs {
    use super::*;

    pub fn public_endpoints() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 100,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_limit: 20,
            window_size_seconds: 60,
            block_duration_seconds: 60,
        }
    }

    pub fn auth_endpoints() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 5,
            requests_per_hour: 50,
            requests_per_day: 200,
            burst_limit: 2,
            window_size_seconds: 60,
            block_duration_seconds: 900, // 15 minutes
        }
    }

    pub fn api_endpoints() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_limit: 10,
            window_size_seconds: 60,
            block_duration_seconds: 300,
        }
    }

    pub fn admin_endpoints() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 100,
            requests_per_hour: 2000,
            requests_per_day: 20000,
            burst_limit: 25,
            window_size_seconds: 60,
            block_duration_seconds: 180,
        }
    }

    pub fn financial_endpoints() -> RateLimitConfig {
        RateLimitConfig {
            requests_per_minute: 10,
            requests_per_hour: 100,
            requests_per_day: 500,
            burst_limit: 3,
            window_size_seconds: 60,
            block_duration_seconds: 1800, // 30 minutes
        }
    }
}
