

// src/handlers/wallet.rs - Updated with SIWE best practices for 2025
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
};
use std::string::String;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use tracing::{info, warn, error};
use std::str::FromStr;
use axum::response::IntoResponse;
use serde_json::json;
use hex;
use k256::{ecdsa::{RecoveryId, Signature, VerifyingKey}, elliptic_curve::sec1::ToEncodedPoint};
use sha3::{Digest, Keccak256};

// Import your database queries and auth utilities
use crate::database::wallet_queries::{find_or_create_wallet_user, update_wallet_login_time};
use crate::handlers::auth::generate_jwt_token;
use crate::models::{User, UserRole, UserStatus};
use crate::AppState;

// Request/Response types following SIWE EIP-4361 standard
#[derive(Deserialize)]
pub struct NonceRequest {
    pub address: String,
}

#[derive(Serialize)]
pub struct NonceResponse {
    pub success: bool,
    pub nonce: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct VerifySignatureRequest {
    pub address: String,
    pub signature: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct WalletAuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user: Option<UserInfo>,
    pub expires_at: Option<DateTime<Utc>>,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub role: String,
    pub wallet_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Nonce storage structure
#[derive(Debug, Clone)]
pub struct NonceData {
    pub nonce: String,
    pub created_at: DateTime<Utc>,
    pub address: String,
    pub domain: String,
}

// In-memory nonce store (consider using Redis for production)
pub type NonceStore = Arc<RwLock<HashMap<String, NonceData>>>;

// SIWE Message structure following EIP-4361
#[derive(Debug)]
pub struct SiweMessage {
    pub domain: String,
    pub address: String,
    pub statement: Option<String>,
    pub uri: String,
    pub version: String,
    pub chain_id: u64,
    pub nonce: String,
    pub issued_at: DateTime<Utc>,
    pub expiration_time: Option<DateTime<Utc>>,
    pub not_before: Option<DateTime<Utc>>,
    pub request_id: Option<String>,
    pub resources: Vec<String>,
}

impl SiweMessage {
    pub fn to_message(&self) -> String {
        let mut message = format!(
            "{} wants you to sign in with your Ethereum account:\n{}\n\n",
            self.domain, self.address
        );

        if let Some(statement) = &self.statement {
            message.push_str(&format!("{}\n\n", statement));
        }

        message.push_str(&format!("URI: {}\n", self.uri));
        message.push_str(&format!("Version: {}\n", self.version));
        message.push_str(&format!("Chain ID: {}\n", self.chain_id));
        message.push_str(&format!("Nonce: {}\n", self.nonce));
        message.push_str(&format!("Issued At: {}\n", self.issued_at.to_rfc3339()));

        if let Some(expiration_time) = &self.expiration_time {
            message.push_str(&format!("Expiration Time: {}\n", expiration_time.to_rfc3339()));
        }

        if let Some(not_before) = &self.not_before {
            message.push_str(&format!("Not Before: {}\n", not_before.to_rfc3339()));
        }

        if let Some(request_id) = &self.request_id {
            message.push_str(&format!("Request ID: {}\n", request_id));
        }

        if !self.resources.is_empty() {
            message.push_str("Resources:\n");
            for resource in &self.resources {
                message.push_str(&format!("- {}\n", resource));
            }
        }

        message
    }
}

// Helper functions
fn generate_nonce() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    const NONCE_LEN: usize = 32;
    
    let mut rng = rand::thread_rng();
    
    (0..NONCE_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn normalize_address(address: &str) -> String {
    address.to_lowercase().trim().to_string()
}

fn create_siwe_message(address: &str, nonce: &str, domain: &str, chain_id: u64) -> SiweMessage {
    let now = Utc::now();
    
    SiweMessage {
        domain: domain.to_string(),
        address: address.to_string(),
        statement: Some("Sign in to TokenPlatform to access your account.".to_string()),
        uri: format!("https://{}", domain),
        version: "1".to_string(),
        chain_id,
        nonce: nonce.to_string(),
        issued_at: now,
        expiration_time: Some(now + Duration::minutes(30)), // 30-minute expiration
        not_before: None,
        request_id: None,
        resources: vec![
            format!("https://{}/terms", domain),
            format!("https://{}/privacy", domain),
        ],
    }
}

async fn cleanup_expired_nonces(nonce_store: &NonceStore) {
    let mut store = nonce_store.write().await;
    let cutoff = Utc::now() - Duration::minutes(30); // 30 minute expiry
    
    store.retain(|_, nonce_data| nonce_data.created_at > cutoff);
}

// Ethereum signature verification
fn verify_eth_signature(signature: &str, message: &str, expected_address: &str) -> bool {
    // Remove 0x prefix if present
    let signature = signature.strip_prefix("0x").unwrap_or(signature);
    
    // Parse signature
    if signature.len() != 130 { // 65 bytes * 2 hex chars = 130
        return false;
    }
    
    let signature_bytes = match hex::decode(signature) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    
    if signature_bytes.len() != 65 {
        return false;
    }
    
    // Split signature into r, s, v components
    let r = &signature_bytes[0..32];
    let s = &signature_bytes[32..64];
    let v = signature_bytes[64];
    
    // Ethereum uses recovery ID (v - 27) for legacy transactions
    let recovery_id = if v >= 27 { v - 27 } else { v };
    
    // Create the message hash (Ethereum signed message format)
    let prefixed_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
    let message_hash = Keccak256::digest(prefixed_message.as_bytes());
    
    // Recover the public key
    let recovery_id = match RecoveryId::try_from(recovery_id) {
        Ok(id) => id,
        Err(_) => return false,
    };
    
    // Construct signature for recovery
    let mut sig_bytes = [0u8; 64];
    sig_bytes[0..32].copy_from_slice(r);
    sig_bytes[32..64].copy_from_slice(s);
    
    let signature = match Signature::from_bytes(&sig_bytes.into()) {
        Ok(sig) => sig,
        Err(_) => return false,
    };
    
    let recovered_key = match VerifyingKey::recover_from_prehash(&message_hash, &signature, recovery_id) {
        Ok(key) => key,
        Err(_) => return false,
    };
    
    // Convert recovered public key to Ethereum address
    let public_key = recovered_key.to_encoded_point(false);
    let public_key_bytes = &public_key.as_bytes()[1..]; // Remove the 0x04 prefix
    
    let address_hash = Keccak256::digest(public_key_bytes);
    let recovered_address = format!("0x{}", hex::encode(&address_hash[12..]));
    
    // Compare with expected address
    recovered_address.to_lowercase() == expected_address.to_lowercase()
}

// Create nonce store
pub fn create_nonce_store() -> NonceStore {
    Arc::new(RwLock::new(HashMap::new()))
}

// Wallet nonce endpoint - following SIWE standard
pub async fn get_wallet_nonce(
    Extension(nonce_store): Extension<NonceStore>,
    Json(request): Json<NonceRequest>,
) -> Result<ResponseJson<NonceResponse>, StatusCode> {
    info!("SIWE nonce request for address: {}", request.address);
    
    // Clean up expired nonces first
    cleanup_expired_nonces(&nonce_store).await;
    
    // Validate address format
    let address = normalize_address(&request.address);
    
    if address.len() != 42 || !address.starts_with("0x") {
        warn!("Invalid wallet address format: {}", address);
        return Ok(ResponseJson(NonceResponse {
            success: false,
            nonce: None,
            message: None,
            error: Some("Invalid wallet address format. Address must be 42 characters starting with 0x".to_string()),
        }));
    }
    
    // Generate new nonce
    let nonce = generate_nonce();
    let domain = "tokenplatform.local"; // Replace with your actual domain
    let chain_id = 1; // Ethereum mainnet, adjust as needed
    
    // Create SIWE message
    let siwe_message = create_siwe_message(&address, &nonce, domain, chain_id);
    let message_text = siwe_message.to_message();
    
    let nonce_data = NonceData {
        nonce: nonce.clone(),
        created_at: Utc::now(),
        address: address.clone(),
        domain: domain.to_string(),
    };
    
    // Store nonce
    {
        let mut store = nonce_store.write().await;
        store.insert(address.clone(), nonce_data);
    }
    
    info!("Generated SIWE nonce for address: {}", address);
    
    Ok(ResponseJson(NonceResponse {
        success: true,
        nonce: Some(nonce),
        message: Some(message_text),
        error: None,
    }))
}

// Verify wallet signature and authenticate - following SIWE standard
pub async fn verify_wallet_signature(
    State(state): State<AppState>,
    Extension(nonce_store): Extension<NonceStore>,
    Json(request): Json<VerifySignatureRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let address = normalize_address(&request.address);
    info!("SIWE signature verification request for address: {}", address);
    
    // Validate nonce exists and is not expired
    let nonce_data = {
        let store = nonce_store.read().await;
        store.get(&address).cloned()
    };
    
    let nonce_data = match nonce_data {
        Some(data) => {
            // Check if nonce is expired (30 minutes)
            if Utc::now() - data.created_at > Duration::minutes(30) {
                warn!("Expired nonce for address: {}", address);
                return Ok(Json(json!({
                    "success": false,
                    "error": "Nonce expired. Please request a new one."
                })));
            }
            data
        }
        None => {
            warn!("Invalid or missing nonce for address: {}", address);
            return Ok(Json(json!({
                "success": false,
                "error": "Invalid or expired nonce. Please request a new one."
            })));
        }
    };
    
    // Parse and validate SIWE message format
    if !request.message.contains(&format!("Nonce: {}", nonce_data.nonce)) {
        warn!("Invalid SIWE message format for address: {}", address);
        return Ok(Json(json!({
            "success": false,
            "error": "Invalid message format. Please use the exact SIWE message provided."
        })));
    }
    
    // Verify that the message contains the correct address
    if !request.message.contains(&address) {
        warn!("Address mismatch in SIWE message for address: {}", address);
        return Ok(Json(json!({
            "success": false,
            "error": "Address mismatch in message."
        })));
    }
    
    // Verify signature
    if !verify_eth_signature(&request.signature, &request.message, &address) {
        warn!("Invalid signature for address: {}", address);
        return Ok(Json(json!({
            "success": false,
            "error": "Invalid signature. Please ensure you're signing with the correct wallet."
        })));
    }
    
    // Remove used nonce to prevent replay attacks
    {
        let mut store = nonce_store.write().await;
        store.remove(&address);
    }
    
    // Find or create user based on wallet address
    match find_or_create_wallet_user(&state.db, &address).await {
        Ok(wallet_user) => {
            // Update login time
            if let Err(e) = update_wallet_login_time(&state.db, wallet_user.id).await {
                error!("Failed to update login time for user {}: {}", wallet_user.id, e);
                // Don't fail the login for this
            }
            
            // Convert WalletUser to User for JWT generation
            let user = User {
                id: wallet_user.id,
                email: wallet_user.email.clone().unwrap_or_else(|| format!("wallet_{}@tokenization.local", &address[2..10])),
                password_hash: String::new(), // Not used for wallet auth
                first_name: wallet_user.first_name.clone(),
                last_name: wallet_user.last_name.clone(),
                phone: None,
                date_of_birth: None,
                nationality: None,
                address: None,
                wallet_address: Some(address.clone()),
                username: Some(wallet_user.username.clone()),
                role: UserRole::from_str(&wallet_user.role).unwrap_or(UserRole::User),
                status: UserStatus::Active, // Wallet users are automatically active
                email_verified: Some(false),
                phone_verified: Some(false),
                two_factor_enabled: Some(false),
                two_factor_secret: None,
                last_login: Some(Utc::now()),
                login_attempts: Some(0),
                locked_until: None,
                reset_token: None,
                reset_token_expires: None,
                verification_token: None,
                verification_token_expires: None,
                created_at: wallet_user.created_at,
                updated_at: wallet_user.updated_at,
            };
            
            // Generate JWT token
            match generate_jwt_token(&user, &state.config.jwt.secret) {
                Ok(token) => {
                    let expires_at = Utc::now() + Duration::hours(24);
                    
                    info!("SIWE authentication successful for address: {}", address);
                    
                    // Return JSON structure that matches frontend expectations
                    let response = json!({
                        "success": true,
                        "token": token,
                        "user": UserInfo {
                            id: wallet_user.id,
                            email: wallet_user.email.clone().unwrap_or_else(|| format!("wallet_{}@tokenization.local", &address[2..10])),
                            first_name: wallet_user.first_name,
                            last_name: wallet_user.last_name,
                            username: Some(wallet_user.username),
                            role: wallet_user.role,
                            wallet_address: address,
                            created_at: wallet_user.created_at,
                            updated_at: wallet_user.updated_at,
                        },
                        "expires_at": expires_at,
                        "message": "SIWE authentication successful"
                    });
                    
                    Ok(Json(response))
                }
                Err(e) => {
                    error!("Token generation error for user {}: {}", wallet_user.id, e);
                    Ok(Json(json!({
                        "success": false,
                        "error": "Failed to generate authentication token"
                    })))
                }
            }
        }
        Err(e) => {
            error!("Database error during SIWE auth for address {}: {}", address, e);
            Ok(Json(json!({
                "success": false,
                "error": "Database error during authentication"
            })))
        }
    }
}

// Disconnect wallet endpoint
pub async fn disconnect_wallet(
    Extension(nonce_store): Extension<NonceStore>,
) -> Result<impl IntoResponse, StatusCode> {
    // In a real implementation, you might want to:
    // 1. Invalidate the JWT token (add to blacklist)
    // 2. Clear any cached session data
    // 3. Log the disconnect event
    
    info!("Wallet disconnect requested");
    
    Ok(Json(json!({
        "success": true,
        "message": "Wallet disconnected successfully"
    })))
}

// Link wallet to existing user endpoint
pub async fn link_wallet_to_user(
    State(state): State<AppState>,
    Json(request): Json<VerifySignatureRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // This would be used when a user wants to link a wallet to their existing account
    // Implementation would depend on your specific requirements
    
    Ok(Json(json!({
        "success": false,
        "error": "Wallet linking not implemented yet"
    })))
}

// Get wallet info endpoint
pub async fn get_wallet_info(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    // This would return information about the connected wallet
    // Implementation would depend on your specific requirements
    
    Ok(Json(json!({
        "success": false,
        "error": "Wallet info endpoint not implemented yet"
    })))
}