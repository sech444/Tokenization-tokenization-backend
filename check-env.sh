#!/bin/bash

# check-env.sh - Environment Variables Verification Script
# This script checks if all required environment variables are properly set

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
WARNINGS=0

# Function to print status
print_check() {
    local status=$1
    local message=$2
    local details=$3

    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✅ PASS${NC} $message"
        ((PASSED++))
    elif [ "$status" = "FAIL" ]; then
        echo -e "${RED}❌ FAIL${NC} $message"
        if [ -n "$details" ]; then
            echo -e "    ${RED}↳${NC} $details"
        fi
        ((FAILED++))
    elif [ "$status" = "WARN" ]; then
        echo -e "${YELLOW}⚠️  WARN${NC} $message"
        if [ -n "$details" ]; then
            echo -e "    ${YELLOW}↳${NC} $details"
        fi
        ((WARNINGS++))
    fi
}

# Function to check if variable exists and is not empty
check_var() {
    local var_name=$1
    local var_value=$2
    local description=$3
    local required=${4:-true}

    if [ -z "$var_value" ]; then
        if [ "$required" = true ]; then
            print_check "FAIL" "$description" "Variable $var_name is not set or empty"
            return 1
        else
            print_check "WARN" "$description" "Optional variable $var_name is not set"
            return 0
        fi
    else
        print_check "PASS" "$description" "Value: $var_value"
        return 0
    fi
}

# Function to check contract address format
check_contract_address() {
    local var_name=$1
    local address=$2
    local contract_name=$3

    if [ -z "$address" ]; then
        print_check "FAIL" "Contract address check: $contract_name" "Variable $var_name is not set"
        return 1
    fi

    # Check if it's a valid Ethereum address format
    if [[ ! "$address" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
        print_check "FAIL" "Contract address check: $contract_name" "Invalid address format: $address"
        return 1
    fi

    # Check if it's not a zero address
    if [ "$address" = "0x0000000000000000000000000000000000000000" ]; then
        print_check "FAIL" "Contract address check: $contract_name" "Address is zero address (not deployed)"
        return 1
    fi

    print_check "PASS" "Contract address check: $contract_name" "Valid address: $address"
    return 0
}

# Load .env file if it exists
if [ -f ".env" ]; then
    echo -e "${BLUE}Loading environment variables from .env file...${NC}"
    export $(grep -v '^#' .env | xargs)
    echo ""
else
    echo -e "${YELLOW}No .env file found. Checking system environment variables...${NC}"
    echo ""
fi

echo "========================================"
echo "🔍 ENVIRONMENT VARIABLES CHECK"
echo "========================================"
echo ""

echo -e "${BLUE}📋 CORE CONTRACT ADDRESSES${NC}"
echo "----------------------------------------"
check_contract_address "CONTRACT_TOKEN_FACTORY" "$CONTRACT_TOKEN_FACTORY" "TokenFactory"
check_contract_address "CONTRACT_MARKETPLACE" "$CONTRACT_MARKETPLACE" "MarketplaceCore"
check_contract_address "CONTRACT_COMPLIANCE" "$CONTRACT_COMPLIANCE" "ComplianceManager"
check_contract_address "CONTRACT_STAKING" "$CONTRACT_STAKING" "RewardSystem (Staking)"

echo ""
echo -e "${BLUE}🔗 BLOCKCHAIN CONFIGURATION${NC}"
echo "----------------------------------------"
check_var "BLOCKCHAIN_RPC_URL" "$BLOCKCHAIN_RPC_URL" "Blockchain RPC URL"
check_var "BLOCKCHAIN_PRIVATE_KEY" "$BLOCKCHAIN_PRIVATE_KEY" "Blockchain Private Key"
check_var "BLOCKCHAIN_NETWORK" "$BLOCKCHAIN_NETWORK" "Blockchain Network" false

# Validate RPC URL format
if [ -n "$BLOCKCHAIN_RPC_URL" ]; then
    if [[ "$BLOCKCHAIN_RPC_URL" =~ ^https?:// ]]; then
        print_check "PASS" "RPC URL format validation" "Valid HTTP/HTTPS URL"
    else
        print_check "WARN" "RPC URL format validation" "URL should start with http:// or https://"
    fi
fi

# Validate private key format
if [ -n "$BLOCKCHAIN_PRIVATE_KEY" ]; then
    if [[ "$BLOCKCHAIN_PRIVATE_KEY" =~ ^0x[a-fA-F0-9]{64}$ ]]; then
        print_check "PASS" "Private key format validation" "Valid private key format"
    else
        print_check "FAIL" "Private key format validation" "Invalid private key format (should be 0x followed by 64 hex characters)"
    fi
fi

echo ""
echo -e "${BLUE}🗄️  DATABASE CONFIGURATION${NC}"
echo "----------------------------------------"
check_var "DATABASE_URL" "$DATABASE_URL" "Database URL"

# Validate database URL format
if [ -n "$DATABASE_URL" ]; then
    if [[ "$DATABASE_URL" =~ ^postgresql:// ]]; then
        print_check "PASS" "Database URL format validation" "Valid PostgreSQL URL"
    else
        print_check "FAIL" "Database URL format validation" "URL should start with postgresql://"
    fi
fi

echo ""
echo -e "${BLUE}🌐 SERVER CONFIGURATION${NC}"
echo "----------------------------------------"
check_var "SERVER_HOST" "$SERVER_HOST" "Server Host" false
check_var "SERVER_PORT" "$SERVER_PORT" "Server Port" false

# Validate port number
if [ -n "$SERVER_PORT" ]; then
    if [[ "$SERVER_PORT" =~ ^[0-9]+$ ]] && [ "$SERVER_PORT" -ge 1 ] && [ "$SERVER_PORT" -le 65535 ]; then
        print_check "PASS" "Server port validation" "Valid port number: $SERVER_PORT"
    else
        print_check "FAIL" "Server port validation" "Invalid port number (should be 1-65535)"
    fi
fi

echo ""
echo -e "${BLUE}🔐 SECURITY CONFIGURATION${NC}"
echo "----------------------------------------"
check_var "JWT_SECRET" "$JWT_SECRET" "JWT Secret Key"

# Check JWT secret strength
if [ -n "$JWT_SECRET" ]; then
    if [ ${#JWT_SECRET} -ge 32 ]; then
        print_check "PASS" "JWT secret strength check" "Secret is sufficiently long (${#JWT_SECRET} characters)"
    else
        print_check "WARN" "JWT secret strength check" "Secret should be at least 32 characters for production"
    fi

    if [ "$JWT_SECRET" = "your-super-secret-jwt-key-change-this-in-production" ] || \
       [ "$JWT_SECRET" = "your-super-secret-jwt-key-change-this-in-production-immediately" ]; then
        print_check "FAIL" "JWT secret security check" "Using default/template JWT secret - CHANGE THIS!"
    else
        print_check "PASS" "JWT secret security check" "Custom JWT secret configured"
    fi
fi

echo ""
echo -e "${BLUE}🔍 KYC/AML CONFIGURATION${NC}"
echo "----------------------------------------"
check_var "KYC_PROVIDER" "$KYC_PROVIDER" "KYC Provider" false
check_var "KYC_API_KEY" "$KYC_API_KEY" "KYC API Key"
check_var "AML_PROVIDER" "$AML_PROVIDER" "AML Provider" false
check_var "AML_API_KEY" "$AML_API_KEY" "AML API Key"
check_var "AUTO_VERIFICATION" "$AUTO_VERIFICATION" "Auto Verification Setting" false
check_var "VERIFICATION_TIMEOUT_HOURS" "$VERIFICATION_TIMEOUT_HOURS" "Verification Timeout" false

# Validate KYC API key is not using default
if [ -n "$KYC_API_KEY" ]; then
    if [ "$KYC_API_KEY" = "demo-kyc-api-key-for-development" ]; then
        print_check "WARN" "KYC API key security check" "Using demo KYC API key - replace with real API key for production"
    else
        print_check "PASS" "KYC API key security check" "Custom KYC API key configured"
    fi
fi

# Validate AML API key is not using default
if [ -n "$AML_API_KEY" ]; then
    if [ "$AML_API_KEY" = "demo-aml-api-key-for-development" ]; then
        print_check "WARN" "AML API key security check" "Using demo AML API key - replace with real API key for production"
    else
        print_check "PASS" "AML API key security check" "Custom AML API key configured"
    fi
fi

echo ""
echo -e "${BLUE}📱 PUSH NOTIFICATIONS (OPTIONAL)${NC}"
echo "----------------------------------------"
check_var "FIREBASE_KEY" "$FIREBASE_KEY" "Firebase Key for push notifications" false
check_var "APNS_KEY" "$APNS_KEY" "Apple Push Notification Service Key" false
check_var "APNS_KEY_ID" "$APNS_KEY_ID" "APNS Key ID" false
check_var "APNS_TEAM_ID" "$APNS_TEAM_ID" "Apple Team ID" false

echo ""
echo -e "${BLUE}⚙️  OPTIONAL CONFIGURATION${NC}"
echo "----------------------------------------"
check_var "REDIS_URL" "$REDIS_URL" "Redis URL (for caching)" false
check_var "RUST_LOG" "$RUST_LOG" "Rust log level" false
check_var "ENVIRONMENT" "$ENVIRONMENT" "Environment type" false

echo ""
echo "========================================"
echo "📊 SUMMARY"
echo "========================================"

if [ $FAILED -eq 0 ]; then
    if [ $WARNINGS -eq 0 ]; then
        echo -e "${GREEN}🎉 ALL CHECKS PASSED!${NC}"
        echo "Your environment is properly configured."
    else
        echo -e "${GREEN}✅ Core requirements satisfied${NC}"
        echo -e "${YELLOW}⚠️  $WARNINGS warning(s) found${NC} - review optional settings"
    fi
    echo ""
    echo -e "${GREEN}✅ Ready to run: cargo run${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAILED critical error(s) found${NC}"
    if [ $WARNINGS -gt 0 ]; then
        echo -e "${YELLOW}⚠️  $WARNINGS warning(s) also found${NC}"
    fi
    echo ""
    echo -e "${RED}Please fix the errors above before running the application.${NC}"
    echo ""
    echo "Quick fixes:"
    echo "1. Copy and edit environment file: cp .env.template .env"
    echo "2. Source environment setup: source ./setup-env.sh"
    echo "3. Run this check again: ./check-env.sh"
    exit 1
fi
