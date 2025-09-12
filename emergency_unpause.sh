# emergency_unpause.sh - Emergency unpause procedures
emergency_unpause() {
    echo "Initiating emergency unpause..."
    echo "This will restore platform operations"
    read -p "Are you sure systems are secure? (yes/no): " confirm
    
    if [[ $confirm == "yes" ]]; then
        source "deployed_addresses.env"
        
        # Unpause marketplace  
        cast send "$MarketplaceCore" "unpause()" \
            --rpc-url "$POLYGON_RPC_URL" \
            --private-key "$PRIVATE_KEY"
        
        # Unpause tokenizer
        cast send "$AssetTokenizer" "unpause()" \
            --rpc-url "$POLYGON_RPC_URL" \
            --private-key "$PRIVATE_KEY"
        
        echo "✅ Emergency unpause completed"
        echo "Platform operations restored"
    else
        echo "Emergency unpause cancelled"
    fi
}