# =============================================================================
# UPGRADE UTILITIES  
# =============================================================================

# prepare_upgrade.sh - Prepare contract upgrades
prepare_upgrade() {
    local contract_name=$1
    local new_implementation=$2
    
    echo "Preparing upgrade for $contract_name..."
    
    # Compile new implementation
    forge build
    
    # Deploy new implementation
    new_addr=$(forge create "$new_implementation" \
        --rpc-url "$POLYGON_RPC_URL" \
        --private-key "$PRIVATE_KEY" \
        --verify)
    
    echo "New implementation deployed at: $new_addr"
    echo "Queue upgrade transaction in timelock contract"
    echo "Upgrade will be executable after timelock delay"
}