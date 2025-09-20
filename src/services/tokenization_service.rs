// src/services/tokenization_service.rs

use sqlx::PgPool;
use uuid::Uuid;
use tracing::{info, error, warn};
use std::sync::Arc;

use crate::{
    services::blockchain::{BlockchainService, TokenizationRequest, TokenizationResult},
    database::projects::{get_project_by_id, update_project_tokenization},
    models::project::{Project, ProjectStatus},
    utils::errors::{AppError, AppResult},
};

pub struct TokenizationService {
    blockchain: Arc<BlockchainService>,
    db: PgPool,
}

impl TokenizationService {
    pub fn new(blockchain: Arc<BlockchainService>, db: PgPool) -> Self {
        Self { blockchain, db }
    }

    /// Complete tokenization flow: Project -> Asset -> Tokenization
    pub async fn tokenize_project(
        &self,
        project_id: Uuid,
        token_params: TokenizationParams,
    ) -> AppResult<TokenizationResult> {
        // Step 1: Get project from database
        let project = self.get_verified_project(project_id).await?;
        
        // Step 2: Register asset on-chain (if not already done)
        let asset_id = self.ensure_asset_registered(&project).await?;
        
        // Step 3: Approve asset (admin operation)
        self.approve_asset_if_needed(asset_id).await?;
        
        // Step 4: Tokenize via HybridAssetTokenizer
        let tokenization_request = TokenizationRequest {
            project_id,
            asset_id,
            token_name: token_params.name,
            token_symbol: token_params.symbol,
            total_supply: token_params.total_supply,
            decimals: token_params.decimals,
            metadata_uri: token_params.metadata_uri,
        };
        
        let result = self.blockchain.tokenize_asset(tokenization_request).await?;
        
        // Step 5: Update database with tokenization results
        self.update_project_tokenization_status(&project, &result).await?;
        
        info!(
            "Successfully tokenized project {} with token address {:?}",
            project_id, result.token_address
        );
        
        Ok(result)
    }

    /// Verify KYC for a user address
    pub async fn verify_user_kyc(
        &self,
        user_address: &str,
        document_hash: String,
        risk_score: u8,
        jurisdiction: String,
    ) -> AppResult<String> {
        let address = user_address.parse()
            .map_err(|_| AppError::ValidationError("Invalid Ethereum address".to_string()))?;
        
        let tx_hash = self.blockchain
            .verify_kyc(address, document_hash, risk_score, jurisdiction)
            .await?;
        
        Ok(tx_hash)
    }

    /// Check if user is KYC verified
    pub async fn check_kyc_status(&self, user_address: &str) -> AppResult<bool> {
        let address = user_address.parse()
            .map_err(|_| AppError::ValidationError("Invalid Ethereum address".to_string()))?;
        
        let is_verified = self.blockchain.is_kyc_verified(address).await?;
        Ok(is_verified)
    }

    /// Get tokenization status for a project
    pub async fn get_tokenization_status(&self, project_id: Uuid) -> AppResult<TokenizationStatus> {
        let project = get_project_by_id(&self.db, &project_id).await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        if !project.is_tokenized {
            return Ok(TokenizationStatus::NotTokenized);
        }

        let token_address = project.token_contract_address
            .ok_or_else(|| AppError::DatabaseError("Tokenized project missing contract address".to_string()))?;

        // You might want to add more on-chain verification here
        Ok(TokenizationStatus::Tokenized { 
            contract_address: token_address,
            deed_id: None, // You'd store this in DB after tokenization
        })
    }

    // Private helper methods
    
    async fn get_verified_project(&self, project_id: Uuid) -> AppResult<Project> {
        let project = get_project_by_id(&self.db, &project_id).await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        // Ensure project is ready for tokenization
        match project.status {
            ProjectStatus::Active => Ok(project),
            _ => Err(AppError::ValidationError("Project must be active to tokenize".to_string())),
        }
    }

    async fn ensure_asset_registered(&self, project: &Project) -> AppResult<u64> {
        // Check if we already have an asset_id stored (you'd add this field to your Project model)
        // For now, we'll register a new asset each time
        
        let document_hashes = self.generate_document_hashes(project)?;
        let asset_id = self.blockchain.register_asset(project, document_hashes).await?;
        
        // TODO: Store asset_id in your database
        // You might want to add an `asset_id` field to your Project model
        
        Ok(asset_id)
    }

    async fn approve_asset_if_needed(&self, asset_id: u64) -> AppResult<()> {
        // In production, this would be a separate admin operation
        // For demo, we'll auto-approve
        let tx_hash = self.blockchain.approve_asset(asset_id).await?;
        
        info!("Asset {} approved with tx: {}", asset_id, tx_hash);
        Ok(())
    }

    async fn update_project_tokenization_status(
        &self,
        project: &Project,
        result: &TokenizationResult,
    ) -> AppResult<()> {
        let contract_address = format!("{:?}", result.token_address);
        
        update_project_tokenization(&self.db, project.id, &contract_address).await?;
        
        // TODO: Store deed_id and other tokenization metadata
        // You might want to create a separate tokenization_records table
        
        Ok(())
    }

    fn generate_document_hashes(&self, project: &Project) -> AppResult<Vec<String>> {
        let mut hashes = Vec::new();
        
        // Convert your legal_documents to IPFS hashes or document hashes
        if let Some(doc_url) = &project.legal_documents {
            // In production, you'd hash the document content
            // For now, use a simple hash of the URL
            let hash = format!("doc_{}", uuid::Uuid::new_v4());
            hashes.push(hash);
        }
        
        // Add property details hash
        let property_hash = format!("prop_{}", uuid::Uuid::new_v4());
        hashes.push(property_hash);
        
        if hashes.is_empty() {
            hashes.push("default_doc_hash".to_string());
        }
        
        Ok(hashes)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenizationParams {
    pub name: String,
    pub symbol: String,
    pub total_supply: u64,
    pub decimals: u8,
    pub metadata_uri: String,
}

#[derive(Debug, serde::Serialize)]
pub enum TokenizationStatus {
    NotTokenized,
    Tokenized {
        contract_address: String,
        deed_id: Option<u64>,
    },
}

// Add these fields to your Project model in models/project.rs
/*
#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Project {
    // ... existing fields ...
    pub asset_id: Option<i64>,           // On-chain asset ID
    pub deed_id: Option<i64>,            // NFT deed ID
    pub tokenization_tx_hash: Option<String>, // Transaction hash
    pub tokenized_at: Option<chrono::DateTime<chrono::Utc>>, // When tokenized
}
*/