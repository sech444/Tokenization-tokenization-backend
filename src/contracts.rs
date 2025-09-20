use ethers::prelude::*;

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
