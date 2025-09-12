# =============================================================================
# DEPLOYMENT UTILITIES
# =============================================================================

# verify_deployment.sh - Script to verify deployment success
#!/bin/bash

verify_deployment() {
    local network=$1
    local addresses_file="deployed_addresses.env"
    
    echo "Verifying deployment on $network..."
    
    source "$addresses_file"
    
    # Check each contract deployment
    contracts=(
        "AuditTrail=$AuditTrail"
        "AdminGovernance=$AdminGovernance"
        "FeeManager=$FeeManager"
        "ComplianceManager=$ComplianceManager"
        "TokenFactory=$TokenFactory"
        "AssetTokenizer=$AssetTokenizer"
        "RewardSystem=$RewardSystem"
        "MarketplaceCore=$MarketplaceCore"
        "TokenizationPlatformFactory=$TokenizationPlatformFactory"
    )
    
    for contract in "${contracts[@]}"; do
        name=$(echo $contract | cut -d'=' -f1)
        address=$(echo $contract | cut -d'=' -f2)
        
        if [[ -n "$address" ]]; then
            code=$(cast code "$address" --rpc-url "$POLYGON_RPC_URL")
            if [[ "$code" != "0x" ]]; then
                echo "✅ $name deployed at $address"
            else
                echo "❌ $name at $address has no code"
            fi
        else
            echo "⚠️  $name address not found"
        fi
    done
}

# estimate_gas.sh - Gas estimation utility
estimate_deployment_gas() {
    echo "Estimating deployment gas costs..."
    
    # Compile contracts
    forge build
    
    # Get current gas price
    gas_price=$(cast gas-price --rpc-url "$POLYGON_RPC_URL")
    
    echo "Current gas price: $(cast from-wei $gas_price) MATIC"
    
    # Estimate individual contract costs
    contracts=(
        "AuditTrail"
        "AdminGovernance"
        "FeeManager"
        "ComplianceManager"
        "TokenFactory"
        "AssetTokenizer" 
        "RewardSystem"
        "MarketplaceCore"
        "TokenizationPlatformFactory"
    )
    
    total_gas=0
    
    for contract in "${contracts[@]}"; do
        # This is a rough estimation - actual deployment will vary
        size=$(jq -r ".contracts[\"contracts/core/$contract.sol\"].$contract.evm.deployedBytecode.object" out/combined.json | wc -c)
        estimated_gas=$((size * 200))  # Rough estimate
        cost_wei=$((estimated_gas * gas_price))
        cost_matic=$(cast from-wei $cost_wei)
        
        echo "$contract: ~$estimated_gas gas (~$cost_matic MATIC)"
        total_gas=$((total_gas + estimated_gas))
    done
    
    total_cost_wei=$((total_gas * gas_price))
    total_cost_matic=$(cast from-wei $total_cost_wei)
    
    echo "----------------------------------------"
    echo "Total estimated gas: $total_gas"
    echo "Total estimated cost: $total_cost_matic MATIC"
    echo "Note: Add 20-30% buffer for actual deployment"
}