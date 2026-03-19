// src/services/blockchain.rs
use std::sync::Arc;
use ethers::{
    prelude::*,
    contract::abigen,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
    utils::parse_ether,
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    models::project::{Project},
    utils::errors::AppError,
};

// Generate contract bindings
abigen!(
    AssetTokenizer,
    r#"[
        function registerAsset(string calldata name, string calldata description, uint8 assetType, string[] calldata documentHashes, string calldata location) external returns (uint256)
        function approveAsset(uint256 assetId) external
        function assets(uint256 assetId) external view returns (uint256, string memory, string memory, uint8, uint8, uint256, uint256, address, address, string[] memory, string memory, uint256, uint256, string memory)
        function getAssetsByStatus(uint8 status) external view returns (uint256[] memory)
        event AssetRegistered(uint256 indexed assetId, address indexed owner, string name, uint8 assetType)
        event AssetApproved(uint256 indexed assetId, address indexed approver)
    ]"#
);

abigen!(
    HybridAssetTokenizer,
    r#"[
        function tokenizeAsset(uint256 assetId, string calldata tokenName, string calldata tokenSymbol, uint256 totalSupply, uint8 decimals, string calldata metadataURI) external payable returns (uint256, address)
        function getTokenForAsset(uint256 assetId) external view returns (address)
        function isAssetTokenized(uint256 assetId) external view returns (bool)
        event AssetTokenized(uint256 indexed assetId, uint256 indexed deedId, address indexed tokenAddress, address tokenizedBy, uint256 totalValue)
        event DeedMinted(uint256 indexed assetId, uint256 indexed deedId, address indexed owner, string assetName, uint8 assetType)
    ]"#
);

abigen!(
    ComplianceManager,
    r#"[
        function verifyKYC(address user, string calldata documentHash, uint8 riskScore, string calldata jurisdiction) external
        function isKYCVerified(address user) external view returns (bool)
        function isAMLCleared(address user) external view returns (bool)
        event KYCVerified(address indexed user, address indexed verifier, uint256 timestamp)
    ]"#
);

#[derive(Debug, Clone)]
pub struct ContractAddresses {
    pub asset_tokenizer: Address,
    pub hybrid_tokenizer: Address,
    pub compliance_manager: Address,
    pub fee_manager: Address,
    pub token_registry: Address,
    pub contract_marketplace: Address,
}

impl Default for ContractAddresses {
    fn default() -> Self {
        Self {
            asset_tokenizer: "0x68b1d87f95878fe05b998f19b66f4baba5de1aed".parse().unwrap(),
            hybrid_tokenizer: "0x0000000000000000000000000000000000000000".parse().unwrap(),
            compliance_manager: "0x0b306bf915c4d645ff596e518faf3f9669b97016".parse().unwrap(),
            fee_manager: "0x959922be3caee4b8cd9a407cc3ac1c251c2007b1".parse().unwrap(),
            token_registry: "0x9a676e781a523b5d0c0e43731313a708cb607508".parse().unwrap(),
            contract_marketplace: "0x0000000000000000000000000000000000000000".parse().unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenizationRequest {
    pub project_id: Uuid,
    pub asset_id: u64, // On-chain asset ID
    pub token_name: String,
    pub token_symbol: String,
    pub total_supply: u64,
    pub decimals: u8,
    pub metadata_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenizationResult {
    pub deed_id: u64,
    pub token_address: Address,
    pub transaction_hash: String,
}

#[derive(Debug)]
pub struct BlockchainService {
    provider: Arc<Provider<Http>>,
    wallet: LocalWallet,
    contracts: ContractAddresses,
    chain_id: u64,
}


impl BlockchainService {
    pub async fn new(
        rpc_url: &str,
        private_key: &str,
        chain_id: u64,
        contracts: ContractAddresses,
    ) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);

        Ok(Self {
            provider: Arc::new(provider),
            wallet,
            contracts,
            chain_id,
        })
    }

    /// Step 1: Register project as on-chain asset
    pub async fn register_asset(
        &self,
        project: &Project,
        document_hashes: Vec<String>,
    ) -> Result<u64, AppError> {
        // create signer client and keep alive in local variable
        let client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone());
        let client = Arc::new(client);

        let contract = AssetTokenizer::new(self.contracts.asset_tokenizer, client.clone());

        // Map ProjectType to on-chain AssetType enum by using Debug string to be robust to variant name
        let pt_str = format!("{:?}", project.project_type);
        let asset_type: u8 = match pt_str.as_str() {
            "RealEstate" | "REAL_ESTATE" | "Real_Estate" | "real_estate" => 0u8,
            "Business" | "BUSINESS" | "business" => 1u8,
            // default fallback
            _ => 0u8,
        };

        info!("Registering asset for project: {}", project.name);

        // project.location is Option<String> in your model -> use empty string if missing
        let location = project.location.clone().unwrap_or_default();

        // Fixed: project.description is String, not Option<String>
        let description = project.description.clone();

        // Call registerAsset. Map errors to AppError
        let call = contract
            .register_asset(
                project.name.clone(),
                description,
                asset_type,
                document_hashes,
                location,
            );
            
        let pending_tx = call
            .send()
            .await
            .map_err(|e| AppError::BlockchainError(format!("Failed to register asset: {}", e)))?;

        let receipt = pending_tx
            .await
            .map_err(|e| AppError::BlockchainError(format!("Transaction failed: {}", e)))?
            .ok_or_else(|| AppError::BlockchainError("No transaction receipt".to_string()))?;

        // Parse AssetRegistered event to get asset ID
        let asset_id = self.parse_asset_registered_event(&receipt).await?;

        info!("Asset registered with ID: {}", asset_id);
        Ok(asset_id)
    }

    /// Step 2: Approve asset (admin function)
    pub async fn approve_asset(&self, asset_id: u64) -> Result<String, AppError> {
        let client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone());
        let client = Arc::new(client);
        let contract = AssetTokenizer::new(self.contracts.asset_tokenizer, client.clone());

        info!("Approving asset ID: {}", asset_id);

        let call = contract.approve_asset(U256::from(asset_id));
        let pending_tx = call
            .send()
            .await
            .map_err(|e| AppError::BlockchainError(format!("Failed to approve asset: {}", e)))?;

        let receipt = pending_tx
            .await
            .map_err(|e| AppError::BlockchainError(format!("Transaction failed: {}", e)))?
            .ok_or_else(|| AppError::BlockchainError("No transaction receipt".to_string()))?;

        Ok(format!("{:?}", receipt.transaction_hash))
    }

    /// Step 3: Tokenize approved asset via HybridAssetTokenizer
    pub async fn tokenize_asset(
        &self,
        request: TokenizationRequest,
    ) -> Result<TokenizationResult, AppError> {
        let client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone());
        let client = Arc::new(client);
        let contract = HybridAssetTokenizer::new(self.contracts.hybrid_tokenizer, client.clone());

        // Calculate fee (or get it from fee manager) and map errors
        let fee = parse_ether("0.01").map_err(|e| AppError::BlockchainError(e.to_string()))?;

        info!("Tokenizing asset ID: {} for project: {}", request.asset_id, request.project_id);

        let call = contract
            .tokenize_asset(
                U256::from(request.asset_id),
                request.token_name.clone(),
                request.token_symbol.clone(),
                U256::from(request.total_supply),
                request.decimals,
                request.metadata_uri.clone(),
            )
            .value(fee);

        let pending_tx = call
            .send()
            .await
            .map_err(|e| AppError::BlockchainError(format!("Failed to tokenize asset: {}", e)))?;

        let receipt = pending_tx
            .await
            .map_err(|e| AppError::BlockchainError(format!("Transaction failed: {}", e)))?
            .ok_or_else(|| AppError::BlockchainError("No transaction receipt".to_string()))?;

        // Parse AssetTokenized event
        let (deed_id, token_address) = self.parse_tokenized_event(&receipt).await?;

        info!("Asset tokenized successfully. Deed ID: {}, Token: {:?}", deed_id, token_address);

        Ok(TokenizationResult {
            deed_id,
            token_address,
            transaction_hash: format!("{:?}", receipt.transaction_hash),
        })
    }

    /// Verify KYC on-chain
    pub async fn verify_kyc(
        &self,
        user_address: Address,
        document_hash: String,
        risk_score: u8,
        jurisdiction: String,
    ) -> Result<String, AppError> {
        let client = SignerMiddleware::new(self.provider.clone(), self.wallet.clone());
        let client = Arc::new(client);
        let contract = ComplianceManager::new(self.contracts.compliance_manager, client.clone());

        let call = contract.verify_kyc(user_address, document_hash, risk_score, jurisdiction);
        let pending_tx = call
            .send()
            .await
            .map_err(|e| AppError::BlockchainError(format!("KYC verification failed: {}", e)))?;

        let receipt = pending_tx
            .await
            .map_err(|e| AppError::BlockchainError(format!("Transaction failed: {}", e)))?
            .ok_or_else(|| AppError::BlockchainError("No transaction receipt".to_string()))?;

        Ok(format!("{:?}", receipt.transaction_hash))
    }

    /// Check if user is KYC verified
    pub async fn is_kyc_verified(&self, user_address: Address) -> Result<bool, AppError> {
        let contract = ComplianceManager::new(self.contracts.compliance_manager, self.provider.clone());

        let result = contract
            .is_kyc_verified(user_address)
            .call()
            .await
            .map_err(|e| AppError::BlockchainError(format!("KYC check failed: {}", e)))?;

        Ok(result)
    }

    /// Get token address for an asset
    pub async fn get_token_for_asset(&self, asset_id: u64) -> Result<Address, AppError> {
        let contract = HybridAssetTokenizer::new(self.contracts.hybrid_tokenizer, self.provider.clone());

        let token_address = contract
            .get_token_for_asset(U256::from(asset_id))
            .call()
            .await
            .map_err(|e| AppError::BlockchainError(format!("Failed to get token address: {}", e)))?;

        Ok(token_address)
    }

    /// Check if asset is tokenized
    pub async fn is_asset_tokenized(&self, asset_id: u64) -> Result<bool, AppError> {
        let contract = HybridAssetTokenizer::new(self.contracts.hybrid_tokenizer, self.provider.clone());

        let is_tokenized = contract
            .is_asset_tokenized(U256::from(asset_id))
            .call()
            .await
            .map_err(|e| AppError::BlockchainError(format!("Failed to check tokenization status: {}", e)))?;

        Ok(is_tokenized)
    }

    // Helper methods for parsing events
    async fn parse_asset_registered_event(&self, receipt: &TransactionReceipt) -> Result<u64, AppError> {
        for log in &receipt.logs {
            if log.address == self.contracts.asset_tokenizer {
                if let Ok(decoded) = AssetTokenizerEvents::decode_log(&log.clone().into()) {
                    if let AssetTokenizerEvents::AssetRegisteredFilter(event) = decoded {
                        return Ok(event.asset_id.as_u64());
                    }
                }
            }
        }
        Err(AppError::BlockchainError("AssetRegistered event not found".to_string()))
    }

    async fn parse_tokenized_event(&self, receipt: &TransactionReceipt) -> Result<(u64, Address), AppError> {
        for log in &receipt.logs {
            if log.address == self.contracts.hybrid_tokenizer {
                if let Ok(decoded) = HybridAssetTokenizerEvents::decode_log(&log.clone().into()) {
                    if let HybridAssetTokenizerEvents::AssetTokenizedFilter(event) = decoded {
                        return Ok((event.deed_id.as_u64(), event.token_address));
                    }
                }
            }
        }
        Err(AppError::BlockchainError("AssetTokenized event not found".to_string()))
    }
}

// Convert eyre errors (if you use them elsewhere)
impl From<eyre::Error> for AppError {
    fn from(err: eyre::Error) -> Self {
        AppError::BlockchainError(err.to_string())
    }
}