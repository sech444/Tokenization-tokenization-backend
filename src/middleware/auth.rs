
// tokenization-backend/src/middleware/auth.rs

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::Algorithm;
use crate::utils::auth::Claims;
use crate::{
    models::user::{User, UserRole},
    database::users,
    AppState,
    utils::errors::AppError,
};

#[derive(Debug)]
pub struct AuthError {
    pub message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({
            "error": "Authentication failed",
            "message": self.message
        }));
        
        (StatusCode::UNAUTHORIZED, body).into_response()
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AuthError {
            message: err.to_string(),
        }
    }
}

impl From<AppError> for AuthError {
    fn from(err: AppError) -> Self {
        AuthError {
            message: err.to_string(),
        }
    }
}

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new() -> Self {
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key".to_string());
        
        Self { secret }
    }

    pub fn generate_token(&self, user: &User) -> Result<String, jsonwebtoken::errors::Error> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::hours(24); // Token expires in 24 hours

        let claims = Claims {
            user_id: user.id,
            email: user.email.clone(),
            username: user.username.clone().unwrap_or_default(),
            role: user.role.clone().to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )?;

        Ok(token_data.claims)
    }

    pub fn refresh_token(&self, user: &User) -> Result<String, jsonwebtoken::errors::Error> {
        self.generate_token(user)
    }
}

// Custom extractor for authenticated users
pub struct AuthenticatedUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts, 
        state: &AppState
    ) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let TypedHeader(Authorization(bearer)) = 
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AuthError {
                    message: "Missing or invalid authorization header".to_string(),
                })?;

        // Verify the JWT token
        let jwt_service = JwtService::new();
        let claims = jwt_service.verify_token(bearer.token())
            .map_err(|e| AuthError {
                message: format!("Invalid token: {}", e),
            })?;

        // Check if token is expired
        let now = chrono::Utc::now().timestamp() as usize;
        if claims.exp < now {
            return Err(AuthError {
                message: "Token has expired".to_string(),
            });
        }

        // Fetch user from database using claims.user_id
        let user = users::get_user_by_id(&state.db, &claims.user_id)
            .await
            .map_err(|e| AuthError {
                message: format!("Database error: {}", e),
            })?
            .ok_or_else(|| AuthError {
                message: "User not found".to_string(),
            })?;

        // Check if user is active
        match user.status {
            crate::models::user::UserStatus::Suspended => {
                return Err(AuthError {
                    message: "Account is suspended".to_string(),
                });
            },
            crate::models::user::UserStatus::Inactive => {
                return Err(AuthError {
                    message: "Account is inactive".to_string(),
                });
            },
            _ => {}
        }

        Ok(AuthenticatedUser(user))
    }
}

// Admin-only extractor
pub struct RequireAdmin(pub User);

#[async_trait]
impl FromRequestParts<AppState> for RequireAdmin {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts, 
        state: &AppState
    ) -> Result<Self, Self::Rejection> {
        let AuthenticatedUser(user) = AuthenticatedUser::from_request_parts(parts, state)
            .await?;

        match user.role {
            UserRole::Admin => Ok(RequireAdmin(user)),
            _ => Err(AuthError {
                message: "Admin access required".to_string(),
            }),
        }
    }
}
