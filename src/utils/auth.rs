use crate::utils::errors::AppError;
use axum::async_trait;
use axum::RequestPartsExt;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts},
    
};
use bcrypt::verify; 
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub role: String,
    pub exp: usize, // Expiration time (Unix timestamp)
    pub iat: usize, // Issued at (Unix timestamp)
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, username: String, role: String) -> Self {
        let now = chrono::Utc::now().timestamp() as usize;
        let exp = now + 24 * 60 * 60; // 24 hours from now

        Claims {
            user_id,
            email,
            username,
            role,
            exp,
            iat: now,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp() as usize;
        self.exp < now
    }
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }

    pub fn generate_token(&self, claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
        encode(&Header::default(), claims, &self.encoding_key)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(token_data.claims)
    }
}

// Axum extractor for JWT authentication
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::AuthenticationFailed("Missing authorization header".into()))?;

        // For now, we'll do a simple token validation
        // In a real implementation, you would decode and validate the JWT
        let token = bearer.token();

        // Mock validation - in production, decode the JWT properly
        if token.is_empty() {
            return Err(AppError::InvalidToken("Token cannot be empty".to_string()));
        }

        // Mock claims for development
        Ok(Claims {
            user_id: Uuid::new_v4(),
            email: "user@example.com".to_string(),
            username: "testuser".to_string(),
            role: "user".to_string(),
            exp: (chrono::Utc::now().timestamp() + 24 * 60 * 60) as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        })
    }
}

// pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
//     bcrypt::hash(password, bcrypt::DEFAULT_COST)
// }
/// Hash a password using bcrypt
pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::EncryptionError(format!("Failed to hash password: {}", e)))
}


pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let claims = Claims::new(
            Uuid::new_v4(),
            "test@example.com".to_string(),
            "testuser".to_string(),
            "user".to_string(),
        );

        assert!(!claims.is_expired());
        assert!(!claims.is_admin());
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn test_admin_role() {
        let mut claims = Claims::new(
            Uuid::new_v4(),
            "admin@example.com".to_string(),
            "admin".to_string(),
            "admin".to_string(),
        );
        claims.role = "admin".to_string();

        assert!(claims.is_admin());
    }

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("Failed to hash password");

        assert_ne!(password, hash);
        assert!(verify_password(password, &hash).expect("Failed to verify password"));
        assert!(!verify_password("wrong_password", &hash).expect("Failed to verify password"));
    }
}
