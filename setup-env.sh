#!/bin/bash

# setup-env.sh - Environment setup script for Tokenization Platform Backend
# This script exports all necessary environment variables for development

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Setting up environment variables for Tokenization Platform...${NC}"

# Core Contract Addresses (from deployed contracts)
export CONTRACT_TOKEN_FACTORY=0x7a2088a1bFc9d81c55368AE168C2C02570cB814F
export CONTRACT_MARKETPLACE=0x09635F643e140090A9A8Dcd712eD6285858ceBef
export CONTRACT_COMPLIANCE=0x4ed7c70F96B99c776995fB64377f0d4aB3B0e1C1
export CONTRACT_STAKING=0xa85233C63b9Ee964Add6F2cffe00Fd84eb32338f

# Additional Contract Addresses
export CONTRACT_ASSET_TOKENIZER=0x4A679253410272dd5232B3Ff7cF5dbB88f295319
export CONTRACT_FEE_MANAGER=0x322813Fd9A801c5507c9de605d63CEA4f2CE6c44
export CONTRACT_PLATFORM_DEPLOYER=0xa82fF9aFd8f496c3d6ac40E2a0F282E47488CFc9
export CONTRACT_REGISTRY=0x59b670e9fA9D0A427751Af201D676719a970857b

# Blockchain Configuration
export BLOCKCHAIN_NETWORK=Amoy
export BLOCKCHAIN_RPC_URL=https://polygon-amoy.g.alchemy.com/v2/JiMaxR6ljFiTZuXrGnZcV
export BLOCKCHAIN_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
export BLOCKCHAIN_GAS_LIMIT=3000000
export BLOCKCHAIN_GAS_PRICE=20000000000

# Database Configuration
export DATABASE_URL="postgres://8cfdde2776b9e0f0662488be70e876ae2894f1a3cd06994b390acf38197d3c4f:sk_qLilo5f9OziArWt4hzbIq@db.prisma.io:5432/?sslmode=require"
export POSTGRES_URL="postgres://8cfdde2776b9e0f0662488be70e876ae2894f1a3cd06994b390acf38197d3c4f:sk_qLilo5f9OziArWt4hzbIq@db.prisma.io:5432/?sslmode=require"
export PRISMA_DATABASE_URL="prisma+postgres://accelerate.prisma-data.net/?api_key=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqd3RfaWQiOjEsInNlY3VyZV9rZXkiOiJza19xTGlsbzVmOU96aUFyV3Q0aHpiSXEiLCJhcGlfa2V5IjoiMDFLM0dOSFcyVFQ0SkFURzlWWDRUMFI5OUgiLCJ0ZW5hbnRfaWQiOiI4Y2ZkZGUyNzc2YjllMGYwNjYyNDg4YmU3MGU4NzZhZTI4OTRmMWEzY2QwNjk5NGIzOTBhY2YzODE5N2QzYzRmIiwiaW50ZXJuYWxfc2VjcmV0IjoiYmNjNmJmYzYtMjZlZC00YjVlLTljZTctOTMyMzQwMjc2MGIzIn0.9u3ynMWaiif4pJocszlIoC8aOqr489uq5hmZBPBf_R4"


# Server Configuration
export SERVER_HOST=127.0.0.1
export SERVER_PORT=8080
export CORS_ORIGINS=http://localhost:3000,http://localhost:3001,http://127.0.0.1:3000

# JWT Configuration
export JWT_SECRET=your-super-secret-jwt-key-change-this-in-production-$(date +%s)
export JWT_EXPIRES_IN=86400

# Redis Configuration (for session/cache)
export REDIS_URL=redis://127.0.0.1:6379

# Environment and Logging
export RUST_LOG=debug,tokenization_platform=trace
export RUST_BACKTRACE=1
export ENVIRONMENT=development

# Security Configuration
export BCRYPT_COST=10
export SESSION_TIMEOUT_MINUTES=60
export MAX_LOGIN_ATTEMPTS=5
export LOCKOUT_DURATION_MINUTES=15

# Rate Limiting
export RATE_LIMIT_REQUESTS=1000
export RATE_LIMIT_WINDOW_SECONDS=60

# KYC/AML Configuration
export KYC_PROVIDER=jumio
export KYC_API_KEY=demo-kyc-api-key-for-development
export AML_PROVIDER=chainalysis
export AML_API_KEY=demo-aml-api-key-for-development
export AUTO_VERIFICATION=false
export VERIFICATION_TIMEOUT_HOURS=72

# Push Notifications Configuration
export FIREBASE_KEY=your-firebase-server-key-for-push-notifications
export APNS_KEY=your-apns-key-for-ios-push
export APNS_KEY_ID=your-apns-key-id
export APNS_TEAM_ID=your-apple-team-id

# Email Configuration (for development)
export SMTP_HOST=smtp.gmail.com
export SMTP_PORT=587
export SMTP_USERNAME=your-email@gmail.com
export SMTP_PASSWORD=your-app-password
export SMTP_USE_TLS=true
export FROM_EMAIL=noreply@tokenization-platform.local
export FROM_NAME=Tokenization Platform

echo -e "${GREEN}✅ Environment variables exported successfully!${NC}"
echo ""
echo "Contract Addresses:"
echo "  Token Factory: $CONTRACT_TOKEN_FACTORY"
echo "  Marketplace: $CONTRACT_MARKETPLACE"
echo "  Compliance: $CONTRACT_COMPLIANCE"
echo "  Staking (RewardSystem): $CONTRACT_STAKING"
echo ""
echo "Blockchain:"
echo "  Network: $BLOCKCHAIN_NETWORK"
echo "  RPC URL: $BLOCKCHAIN_RPC_URL"
echo ""
echo "Database:"
echo "  URL: $DATABASE_URL"
echo "  Postgres URL: $POSTGRES_URL"
echo "  Prisma URL: $PRISMA_DATABASE_URL"
echo ""
echo "Server:"
echo "  Host: $SERVER_HOST"
echo "  Port: $SERVER_PORT"
echo ""
echo -e "${BLUE}To use these variables in your current shell session, run:${NC}"
echo "  source ./setup-env.sh"
echo ""
echo -e "${BLUE}To start the application after sourcing:${NC}"
echo "  cargo run"
