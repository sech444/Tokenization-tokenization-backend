# Quick Start Guide

Get the Tokenization Platform Backend running in under 5 minutes.

## Prerequisites

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **PostgreSQL** (13+): Running on `localhost:5432`
- **Node.js** (18+): For Foundry/Anvil blockchain
- **Git**: For cloning the repository

## 🚀 Quick Setup (Automated)

```bash
# 1. Navigate to backend directory
cd tokenization-backend

# 2. Setup environment variables
source ./setup-env.sh

# 3. Verify configuration
./check-env.sh

# 4. Start the application
cargo run
```

✅ **That's it!** Your server should be running at `http://localhost:8080/health`

## 📋 Manual Setup (If Needed)

### 1. Environment Configuration

Copy and configure the environment file:
```bash
cp .env.template .env
```

The key variables that **must** be set:
```bash
# Contract Addresses (Already configured for local development)
CONTRACT_TOKEN_FACTORY=0x7a2088a1bFc9d81c55368AE168C2C02570cB814F
CONTRACT_MARKETPLACE=0x09635F643e140090A9A8Dcd712eD6285858ceBef
CONTRACT_COMPLIANCE=0x4ed7c70F96B99c776995fB64377f0d4aB3B0e1C1
CONTRACT_STAKING=0xa85233C63b9Ee964Add6F2cffe00Fd84eb32338f    # RewardSystem contract

# KYC/AML (Required)
KYC_API_KEY=demo-kyc-api-key-for-development
AML_API_KEY=demo-aml-api-key-for-development

# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/tokenization_platform

# Blockchain
BLOCKCHAIN_RPC_URL=http://127.0.0.1:8545
BLOCKCHAIN_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

### 2. Database Setup

```bash
# Create database
createdb tokenization_platform

# Run migrations (automatic when starting the app)
# sqlx migrate run
```

### 3. Start Local Blockchain (Optional)

```bash
# In a separate terminal
anvil --host 0.0.0.0 --port 8545
```

### 4. Run the Application

```bash

source ./setup-env.sh && cargo run
cargo run
```

## 🔍 Verification

Test the API endpoints:
```bash
# Health check
curl http://localhost:8080/health

# Should return: "Tokenization Platform API is running"
```

## 📁 Project Structure

```
tokenization-backend/
├── .env                    # Main environment file
├── setup-env.sh           # Environment setup script
├── check-env.sh           # Environment validation
├── start-dev.sh           # Complete development startup
├── src/
│   ├── main.rs            # Application entry point
│   ├── config.rs          # Configuration management
│   ├── handlers/          # API route handlers
│   ├── services/          # Business logic
│   └── database/          # Database queries and migrations
├── contracts/             # Smart contracts (Solidity)
└── migrations/           # Database migrations
```

## 🛠 Common Issues & Solutions

### Error: `MissingStakingContract`
**Solution**: The staking functionality is handled by the `RewardSystem` contract.
```bash
export CONTRACT_STAKING=0xa85233C63b9Ee964Add6F2cffe00Fd84eb32338f
```

### Error: `MissingKycApiKey`
**Solution**: Set the KYC and AML API keys:
```bash
export KYC_API_KEY=demo-kyc-api-key-for-development
export AML_API_KEY=demo-aml-api-key-for-development
```

### Error: Database connection failed
**Solution**: Start PostgreSQL and create the database:
```bash
sudo systemctl start postgresql  # Linux
# or
brew services start postgresql   # macOS

createdb tokenization_platform
```

### Error: Contract deployment addresses not found
**Solution**: The provided addresses are for local development. If you redeploy contracts, update the contract addresses in `.env`.

### Port 8080 already in use
**Solution**: Change the port:
```bash
export SERVER_PORT=8081
```

## 🔧 Development Workflow

### Daily Development
```bash
# Quick start
source ./setup-env.sh && cargo run

# With full checks
./start-dev.sh
```

### After Code Changes
```bash
# Just rebuild and run
cargo run

# Full restart with checks
./start-dev.sh
```

### Environment Debugging
```bash
# Check all environment variables
./check-env.sh

# View current config
env | grep -E "(CONTRACT_|DATABASE_|SERVER_)"
```

## 🎯 API Endpoints

Once running, the API provides:

### Public Endpoints
- `GET /health` - Health check
- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login

### Protected Endpoints (require JWT)
- `GET /api/users/me` - Current user profile
- `GET /api/tokens` - List tokens
- `POST /api/tokens` - Create token
- `GET /api/projects` - List projects
- `GET /api/marketplace/listings` - Marketplace listings

### Admin Endpoints
- `GET /api/admin/users` - List all users
- `GET /api/admin/kyc/pending` - Pending KYC verifications

## 📊 Monitoring

### Application Logs
```bash
# View logs with timestamp
cargo run 2>&1 | while IFS= read -r line; do printf '[%s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$line"; done
```

### Database Status
```bash
# Check connection
psql -d tokenization_platform -c "SELECT version();"
```

### Blockchain Status
```bash
# Check if Anvil is running
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://127.0.0.1:8545
```

## 🚦 Next Steps

1. **Frontend Setup**: Start the React frontend in `../tokenization-frontend`
2. **Smart Contract Interaction**: Use the deployed contracts for tokenization
3. **Production Setup**: Configure real KYC/AML providers and secure credentials
4. **Testing**: Run the test suite with `cargo test`

## 📞 Support

If you encounter issues:
1. Run `./check-env.sh` to validate your environment
2. Check application logs for detailed error messages
3. Verify all external services (PostgreSQL, Anvil) are running
4. Ensure all contract addresses are correctly deployed

---

**🎉 You're ready to tokenize assets!** The platform is now running with smart contract integration, database persistence, and a comprehensive API.