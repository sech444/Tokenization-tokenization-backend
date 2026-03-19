# =============================================================================
# MONITORING SETUP
# =============================================================================

# monitor_deployment.sh - Post-deployment monitoring
setup_monitoring() {
    echo "Setting up post-deployment monitoring..."
    
    source "deployed_addresses.env"
    
    # Create monitoring configuration
    cat > monitoring_config.json << EOF
{
  "contracts": {
    "AuditTrail": {
      "address": "$AuditTrail",
      "events": ["LogCreated", "LogUpdated"],
      "alerts": ["high_gas_usage", "failed_transactions"]
    },
    "AdminGovernance": {
      "address": "$AdminGovernance", 
      "events": ["RoleGranted", "RoleRevoked", "ProposalCreated"],
      "alerts": ["admin_changes", "governance_actions"]
    },
    "ComplianceManager": {
      "address": "$ComplianceManager",
      "events": ["UserVerified", "UserSuspended", "ComplianceViolation"],
      "alerts": ["compliance_violations", "suspicious_activity"]
    },
    "MarketplaceCore": {
      "address": "$MarketplaceCore",
      "events": ["OrderCreated", "OrderFilled", "OrderCancelled"],
      "alerts": ["large_trades", "unusual_volume"]
    }0
  },
  "thresholds": {
    "gas_price_alert": 100000000000,
    "large_trade_amount": 100000,
    "unusual_volume_multiplier": 5
  }
}
EOF

    echo "Monitoring configuration created at monitoring_config.json"
    echo "Configure your monitoring service to use this configuration"
}
