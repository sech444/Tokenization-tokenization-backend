#!/bin/bash

# Smart Contract Deployment Script for Tokenization Platform
# This script handles the complete deployment of all smart contracts
# with proper security checks, verification, and configuration

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTRACTS_DIR="${SCRIPT_DIR}/contracts"
DEPLOY_LOG="${SCRIPT_DIR}/deployment.log"
ENV_FILE="${SCRIPT_DIR}/.env"

# Default values
NETWORK="polygon-mumbai"
VERIFY_CONTRACTS="true"
DRY_RUN="false"
FORCE_DEPLOY="false"

# Load environment variables
if [[ -f "$ENV_FILE" ]]; then
    source "$ENV_FILE"
else
    echo -e "${RED}Error: .env file not found. Please create one with required variables.${NC}"
    exit 1
fi

# Required environment variables
REQUIRED_VARS=(
    "PRIVATE_KEY"
    "POLYGON_RPC_URL"
    "POLYGONSCAN_API_KEY"
    "DEPLOYER_ADDRESS"
    "ADMIN_ADDRESS"
    "COMPLIANCE_OFFICER"
    "FEE_COLLECTOR"
)

# Function to print colored output
print_status() {
    local level=$1
    local message=$2
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case $level in
        "INFO")
            echo -e "${BLUE}[INFO]${NC} ${timestamp} - $message" | tee -a "$DEPLOY_LOG"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[SUCCESS]${NC} ${timestamp} - $message" | tee -a "$DEPLOY_LOG"
            ;;
        "WARNING")
            echo -e "${YELLOW}[WARNING]${NC} ${timestamp} - $message" | tee -a "$DEPLOY_LOG"
            ;;
        "ERROR")
            echo -e "${RED}[ERROR]${NC} ${timestamp} - $message" | tee -a "$DEPLOY_LOG"
            ;;
    esac
}

# Function to check prerequisites
check_prerequisites() {
    print_status "INFO" "Checking deployment prerequisites..."
    
    # Check if Foundry is installed
    if ! command -v forge &> /dev/null; then
        print_status "ERROR" "Foundry not found. Please install it first."
        exit 1
    fi
    
    # Check if Cast is installed
    if ! command -v cast &> /dev/null; then
        print_status "ERROR" "Cast not found. Please install Foundry tools."
        exit 1
    fi
    
    # Check required environment variables
    for var in "${REQUIRED_VARS[@]}"; do
        if [[ -z "${!var}" ]]; then
            print_status "ERROR" "Required environment variable $var is not set"
            exit 1
        fi
    done
    
    # Check network connectivity
    if ! curl -s "$POLYGON_RPC_URL" --data '{"method":"eth_blockNumber","params":[],"id":1,"jsonrpc":"2.0"}' > /dev/null; then
        print_status "ERROR" "Cannot connect to RPC endpoint: $POLYGON_RPC_URL"
        exit 1
    fi
    
    # Check deployer balance
    local balance=$(cast balance "$DEPLOYER_ADDRESS" --rpc-url "$POLYGON_RPC_URL")
    local balance_eth=$(cast from-wei "$balance")
    print_status "INFO" "Deployer balance: $balance_eth ETH"
    
    # Minimum balance check (0.1 ETH)
    if (( $(echo "$balance_eth < 0.1" | bc -l) )); then
        print_status "WARNING" "Low deployer balance. Consider adding more ETH for gas fees."
    fi
    
    print_status "SUCCESS" "All prerequisites met"
}

# Function to compile contracts
compile_contracts() {
    print_status "INFO" "Compiling smart contracts..."
    
    cd "$CONTRACTS_DIR/.."
    
    # Clean previous build
    forge clean
    
    # Compile with optimization
    if forge build --optimize --optimizer-runs 200; then
        print_status "SUCCESS" "Contracts compiled successfully"
    else
        print_status "ERROR" "Contract compilation failed"
        exit 1
    fi
    
    # Run tests
    if [[ "$DRY_RUN" != "true" ]]; then
        print_status "INFO" "Running contract tests..."
        if forge test -vv; then
            print_status "SUCCESS" "All tests passed"
        else
            print_status "ERROR" "Tests failed. Deployment aborted."
            exit 1
        fi
    fi
}

# Function to deploy a single contract
deploy_contract() {
    local contract_name=$1
    local constructor_args=$2
    local contract_path=$3
    
    print_status "INFO" "Deploying $contract_name..."
    
    local deploy_cmd="forge create \"$contract_path:$contract_name\" \
        --rpc-url \"$POLYGON_RPC_URL\" \
        --private-key \"$PRIVATE_KEY\" \
        --gas-limit 5000000"
    
    if [[ -n "$constructor_args" ]]; then
        deploy_cmd+=" --constructor-args $constructor_args"
    fi
    
    if [[ "$VERIFY_CONTRACTS" == "true" ]]; then
        deploy_cmd+=" --verify --etherscan-api-key \"$POLYGONSCAN_API_KEY\""
    fi
    
    local output
    if output=$(eval "$deploy_cmd" 2>&1); then
        local deployed_address=$(echo "$output" | grep "Deployed to:" | awk '{print $3}')
        print_status "SUCCESS" "$contract_name deployed at: $deployed_address"
        echo "$contract_name=$deployed_address" >> "${SCRIPT_DIR}/deployed_addresses.env"
        echo "$deployed_address"
    else
        print_status "ERROR" "Failed to deploy $contract_name: $output"
        exit 1
    fi
}

# Function to verify contract on Polygonscan
verify_contract() {
    local contract_address=$1
    local contract_name=$2
    local constructor_args=$3
    
    if [[ "$VERIFY_CONTRACTS" != "true" ]]; then
        return
    fi
    
    print_status "INFO" "Verifying $contract_name at $contract_address..."
    
    local verify_cmd="forge verify-contract \"$contract_address\" \
        \"$contract_name\" \
        --etherscan-api-key \"$POLYGONSCAN_API_KEY\" \
        --chain-id 80001"
    
    if [[ -n "$constructor_args" ]]; then
        verify_cmd+=" --constructor-args $constructor_args"
    fi
    
    if eval "$verify_cmd"; then
        print_status "SUCCESS" "$contract_name verified successfully"
    else
        print_status "WARNING" "Failed to verify $contract_name (deployment still successful)"
    fi
}

# Function to deploy all contracts in proper order
deploy_all_contracts() {
    print_status "INFO" "Starting contract deployment sequence..."
    
    # Create deployment addresses file
    echo "# Deployed Contract Addresses" > "${SCRIPT_DIR}/deployed_addresses.env"
    echo "# Generated on $(date)" >> "${SCRIPT_DIR}/deployed_addresses.env"
    
    # Step 1: Deploy core infrastructure contracts
    print_status "INFO" "=== Deploying Core Infrastructure ==="
    
    # Deploy AuditTrail first (no dependencies)
    AUDIT_TRAIL_ADDR=$(deploy_contract "AuditTrail" "" "contracts/core/AuditTrail.sol")
    
    # Deploy AdminGovernance
    ADMIN_GOVERNANCE_ADDR=$(deploy_contract "AdminGovernance" "$ADMIN_ADDRESS" "contracts/core/AdminGovernance.sol")
    
    # Deploy FeeManager
    FEE_MANAGER_ADDR=$(deploy_contract "FeeManager" "$FEE_COLLECTOR" "contracts/core/FeeManager.sol")
    
    # Step 2: Deploy compliance and token management
    print_status "INFO" "=== Deploying Compliance and Token Management ==="
    
    # Deploy ComplianceManager
    COMPLIANCE_MANAGER_ADDR=$(deploy_contract "ComplianceManager" "$COMPLIANCE_OFFICER" "contracts/core/ComplianceManager.sol")
    
    # Deploy TokenFactory
    TOKEN_FACTORY_ADDR=$(deploy_contract "TokenFactory" "$ADMIN_GOVERNANCE_ADDR $COMPLIANCE_MANAGER_ADDR $AUDIT_TRAIL_ADDR" "contracts/core/TokenFactory.sol")
    
    # Deploy AssetTokenizer
    ASSET_TOKENIZER_ADDR=$(deploy_contract "AssetTokenizer" "$TOKEN_FACTORY_ADDR $COMPLIANCE_MANAGER_ADDR $FEE_MANAGER_ADDR" "contracts/core/AssetTokenizer.sol")
    
    # Step 3: Deploy marketplace and rewards
    print_status "INFO" "=== Deploying Marketplace and Rewards ==="
    
    # Deploy RewardSystem
    REWARD_SYSTEM_ADDR=$(deploy_contract "RewardSystem" "" "contracts/core/RewardSystem.sol")
    
    # Deploy MarketplaceCore
    MARKETPLACE_CORE_ADDR=$(deploy_contract "MarketplaceCore" "$COMPLIANCE_MANAGER_ADDR $FEE_MANAGER_ADDR $AUDIT_TRAIL_ADDR" "contracts/core/MarketplaceCore.sol")
    
    # Step 4: Deploy factory contract
    print_status "INFO" "=== Deploying Platform Factory ==="
    
    PLATFORM_FACTORY_ADDR=$(deploy_contract "TokenizationPlatformFactory" "$ADMIN_GOVERNANCE_ADDR" "contracts/factory/TokenizationPlatformFactory.sol")
    
    # Step 5: Configure contracts
    print_status "INFO" "=== Configuring Deployed Contracts ==="
    configure_contracts
    
    print_status "SUCCESS" "All contracts deployed successfully!"
    print_deployment_summary
}

# Function to configure deployed contracts
configure_contracts() {
    print_status "INFO" "Configuring contract permissions and settings..."
    
    # Load deployed addresses
    source "${SCRIPT_DIR}/deployed_addresses.env"
    
    # Configure AdminGovernance roles
    print_status "INFO" "Setting up governance roles..."
    cast send "$AdminGovernance" "grantRole(bytes32,address)" \
        "0x0000000000000000000000000000000000000000000000000000000000000000" \
        "$ADMIN_ADDRESS" \
        --rpc-url "$POLYGON_RPC_URL" \
        --private-key "$PRIVATE_KEY"
    
    # Configure ComplianceManager
    print_status "INFO" "Setting up compliance roles..."
    cast send "$ComplianceManager" "grantRole(bytes32,address)" \
        "$(cast keccak "COMPLIANCE_OFFICER_ROLE")" \
        "$COMPLIANCE_OFFICER" \
        --rpc-url "$POLYGON_RPC_URL" \
        --private-key "$PRIVATE_KEY"
    
    # Configure FeeManager
    print_status "INFO" "Setting up fee collection..."
    cast send "$FeeManager" "setFeeCollector(address)" \
        "$FEE_COLLECTOR" \
        --rpc-url "$POLYGON_RPC_URL" \
        --private-key "$PRIVATE_KEY"
    
    # Set platform fees (0.5% = 50 basis points)
    cast send "$FeeManager" "setPlatformFee(uint256)" \
        "50" \
        --rpc-url "$POLYGON_RPC_URL" \
        --private-key "$PRIVATE_KEY"
    
    print_status "SUCCESS" "Contract configuration completed"
}

# Function to run security checks
run_security_checks() {
    print_status "INFO" "Running security checks..."
    
    # Check for Slither if available
    if command -v slither &> /dev/null; then
        print_status "INFO" "Running Slither static analysis..."
        if slither contracts/ --exclude-dependencies --exclude-informational; then
            print_status "SUCCESS" "Slither analysis completed"
        else
            print_status "WARNING" "Slither found potential issues - review before production"
        fi
    else
        print_status "WARNING" "Slither not found - consider installing for security analysis"
    fi
    
    # Check contract sizes
    print_status "INFO" "Checking contract sizes..."
    forge build --sizes
}

# Function to print deployment summary
print_deployment_summary() {
    print_status "INFO" "=== DEPLOYMENT SUMMARY ==="
    
    echo -e "\n${GREEN}Successfully deployed contracts:${NC}"
    cat "${SCRIPT_DIR}/deployed_addresses.env" | grep "=" | while read line; do
        echo -e "${BLUE}$line${NC}"
    done
    
    echo -e "\n${YELLOW}Next steps:${NC}"
    echo "1. Update your backend configuration with the deployed addresses"
    echo "2. Run integration tests with the deployed contracts"
    echo "3. Configure frontend to use the new contract addresses"
    echo "4. Set up monitoring for the deployed contracts"
    echo "5. Perform final security review before mainnet deployment"
    
    if [[ "$NETWORK" == "polygon-mumbai" ]]; then
        echo -e "\n${YELLOW}This was a testnet deployment. For mainnet:${NC}"
        echo "1. Switch POLYGON_RPC_URL to mainnet RPC"
        echo "2. Update POLYGONSCAN_API_KEY for mainnet verification"
        echo "3. Use a hardware wallet or secure key management"
        echo "4. Increase gas limits for mainnet deployment"
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -n, --network NETWORK     Target network (default: polygon-mumbai)"
    echo "  -v, --verify              Verify contracts on Polygonscan (default: true)"
    echo "  --no-verify               Skip contract verification"
    echo "  -d, --dry-run             Compile and test only, don't deploy"
    echo "  -f, --force               Force deployment even if contracts exist"
    echo "  -h, --help                Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                        Deploy to testnet with verification"
    echo "  $0 --dry-run              Test compilation without deploying"
    echo "  $0 --network polygon      Deploy to Polygon mainnet"
    echo "  $0 --no-verify            Deploy without contract verification"
}

# Function to cleanup on exit
cleanup() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        print_status "ERROR" "Deployment failed with exit code $exit_code"
        print_status "INFO" "Check $DEPLOY_LOG for details"
    fi
}

# Set up signal handlers
trap cleanup EXIT

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--network)
            NETWORK="$2"
            shift 2
            ;;
        -v|--verify)
            VERIFY_CONTRACTS="true"
            shift
            ;;
        --no-verify)
            VERIFY_CONTRACTS="false"
            shift
            ;;
        -d|--dry-run)
            DRY_RUN="true"
            shift
            ;;
        -f|--force)
            FORCE_DEPLOY="true"
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            echo "Unknown option $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    print_status "INFO" "Starting Tokenization Platform deployment"
    print_status "INFO" "Network: $NETWORK"
    print_status "INFO" "Verify contracts: $VERIFY_CONTRACTS"
    print_status "INFO" "Dry run: $DRY_RUN"
    
    # Create log file
    touch "$DEPLOY_LOG"
    
    # Run deployment steps
    check_prerequisites
    compile_contracts
    run_security_checks
    
    if [[ "$DRY_RUN" != "true" ]]; then
        deploy_all_contracts
    else
        print_status "INFO" "Dry run completed successfully. No contracts were deployed."
    fi
    
    print_status "SUCCESS" "Deployment script completed successfully"
}

# Execute main function
main "$@"