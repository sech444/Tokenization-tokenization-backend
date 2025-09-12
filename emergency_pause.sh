# =============================================================================
# EMERGENCY PROCEDURES
# =============================================================================

# emergency_pause.sh - Emergency pause procedures  
emergency_pause() {
    echo "⚠️  EMERGENCY PAUSE INITIATED ⚠️"
    echo "This will pause all platform operations"
    read -p "Are you sure? (yes/no): " confirm
    
    if [[ $confirm == "yes" ]]; then
        source "deployed_addresses.env"
        
        # Pause marketplace
        cast send "$MarketplaceCore" "pause()" \
            --rpc-url "$POLYGON_RPC_URL" \
            --private-key "$PRIVATE_KEY"
        
        # Pause tokenizer
        cast send "$AssetTokenizer" "pause()" \
            --rpc-url "$POLYGON_RPC_URL" \
            --private-key "$PRIVATE_KEY"
        
        echo "✅ Emergency pause activated"
        echo "All critical functions are now paused"
        echo "Use emergency_unpause.sh to restore operations"
    else
        echo "Emergency pause cancelled"
    fi
}
