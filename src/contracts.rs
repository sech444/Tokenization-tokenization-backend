// tokenization-backend/src/contracts.rs

use ethers::prelude::*;

// This file generates Rust bindings from your compiled smart contract ABIs.
// The paths point to the JSON ABI files created by Foundry in the 'out/' directory.

abigen!(
    TokenRegistry,
    "out/TokenRegistry.sol/TokenRegistry.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    MarketplaceCore,
    "out/MarketplaceCore.sol/MarketplaceCore.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    ComplianceManager,
    "out/ComplianceManager.sol/ComplianceManager.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    TokenFactory,
    "out/TokenFactory.sol/TokenFactory.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

// ===== ADDED MISSING CONTRACT BINDINGS =====

abigen!(
    HybridAssetTokenizer,
    "out/HybridAssetTokenizer.sol/HybridAssetTokenizer.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    AuditTrail,
    "out/AuditTrail.sol/AuditTrail.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    FeeManager,
    "out/FeeManager.sol/FeeManager.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    AdminGovernance,
    "out/AdminGovernance.sol/AdminGovernance.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    RewardSystem,
    "out/RewardSystem.sol/RewardSystem.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    AssetToken,
    "out/AssetToken.sol/AssetToken.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    AssetVerificationGateway,
    "out/AssetVerificationGateway.sol/AssetVerificationGateway.json",
    event_derives(serde::Deserialize, serde::Serialize)
);