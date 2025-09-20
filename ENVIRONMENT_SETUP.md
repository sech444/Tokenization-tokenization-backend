# Environment Setup Guide

## Overview

This guide explains how to properly configure the environment variables for the Tokenization Platform Backend to resolve the `MissingStakingContract` error and ensure all smart contracts are properly configured.

## The Issue

When running `cargo run`, you might encounter:
```
Error: MissingStakingContract
[Finished running. Exit status: 1]
```

This error occurs because the application expects certain environment variables to be set with the deployed smart contract addresses, but they're either missing or pointing to zero addresses.

## Smart Contract Architecture

The platform uses the following core contracts:

| Environment Variable | Contract Name | Purpose | Deployed Address |
|---------------------|---------------|---------|------------------|
| `CONTRACT_TOKEN_FACTORY` | TokenFactory | ERC-20 token creation | `0x7a2088a1bFc9d81c55368AE168C2C02570cB814F` |
| `CONTRACT_MARKETPLACE` | MarketplaceCore | Trading and order book | `0x09635F643e140090A9A8Dcd712eD6285858ceBef` |
| `CONTRACT_COMPLIANCE` | ComplianceManager | KYC/AML enforcement | `0x4ed7c70F96B99c776995fB64377f0d4aB3B0e1C1` |
| `CONTRACT_STAKING` | **RewardSystem** | Staking & rewards | `0xa85233C63b9Ee964Add6F2cffe00Fd84eb32338f` |

> **Important**: The "staking contract" is actually implemented by the `RewardSystem` contract, which handles staking pools, rewards distribution, loyalty points, and referral programs.

## Quick Setup

### Option 1: Automatic Setup (Recommended)

1. **Source the environment setup script:**
   ```bash
   cd tokenization-backend
   source ./setup-env.sh
   ```

2. **Verify the configuration:**
   ```bash
   ./check-env.sh
   ```

3. **Run the application:**
   ```bash
   cargo run
   ```

### Option 2: Manual Setup

1. **Copy the environment template:**
   ```bash
   cd tokenization-backend
   cp .env.template .env
   ```

2. **Edit `.env` file** (the contract addresses should already be correct):
   ```bash
   nano .env
   # or
   code .env
   ```

3. **Verify and run:**
   ```bash
   ./check-env.sh
   cargo run
   ```

## Environment Files Structure

```
tokenization-backend/
├── .env                          # Main environment file (loaded by dotenv)
├── .env.template                 # Template with all variables and documentation
├── .env.development             # Environment-specific file (loaded if RUST_ENV=development)
├── deployed.development.env      # Contract addresses from deployment
├── setup-env.sh                 # Script to export all variables
├── check-env.sh                 # Script to verify environment setup
└── start-dev.sh                 # Complete development startup script
```

## Required Environment Variables

### Core Contract Addresses
```bash
CONTRACT_TOKEN_FACTORY=0x7a2088a1bFc9d81c55368AE168C2C02570cB814F
CONTRACT_MARKETPLACE=0x09635F643e140090A9A8Dcd712eD6285858ceBef
CONTRACT_COMPLIANCE=0x4ed7c70F96B99c776995fB64377f0d4aB3B0e1C1
CONTRACT_STAKING=0xa85233C63b9Ee964Add6F2cffe00Fd84eb32338f
```

### Blockchain Configuration
```bash
BLOCKCHAIN_NETWORK=localhost
BLOCKCHAIN_RPC_URL=http://127.0.0.1:8545
BLOCKCHAIN_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
BLOCKCHAIN_GAS_LIMIT=3000000
BLOCKCHAIN_GAS_PRICE=20000000000
```

### Database
```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/tokenization_platform
```

### Server
```bash
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
CORS_ORIGINS=http://localhost:3000,http://localhost:3001,http://127.0.0.1:3000
```

### Security
```bash
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production-immediately
JWT_EXPIRES_IN=86400
BCRYPT_COST=12
```

## Scripts Explanation

### `setup-env.sh`
- **Purpose**: Exports all required environment variables for the current shell session
- **Usage**: `source ./setup-env.sh`
- **Note**: Changes only affect the current terminal session

### `check-env.sh`
- **Purpose**: Validates that all required environment variables are properly set
- **Usage**: `./check-env.sh`
- **Features**: 
  - Validates contract address formats
  - Checks for zero addresses
  - Verifies database URL format
  - Tests JWT secret strength
  - Color-coded output with pass/fail/warning status

### `start-dev.sh`
- **Purpose**: Complete development startup script
- **Usage**: `./start-dev.sh`
- **Features**:
  - Loads environment variables
  - Checks database connectivity
  - Runs database migrations
  - Verifies blockchain connection
  - Validates contract addresses
  - Builds and starts the application

## Configuration Loading Order

The application loads environment variables in this order:

1. **System environment variables**
2. **Main `.env` file** (via `dotenv::dotenv()`)
3. **Environment-specific file** (via `RUST_ENV` variable, e.g., `.env.development`)

Later sources override earlier ones.

## Troubleshooting

### Issue: MissingStakingContract Error
**Solution**: Ensure `CONTRACT_STAKING` is set to the RewardSystem contract address:
```bash
export CONTRACT_STAKING=0xa85233C63b9Ee964Add6F2cffe00Fd84eb32338f
```

### Issue: Invalid Contract Address
**Solution**: Check that addresses are:
- Valid Ethereum address format (0x + 40 hex characters)
- Not zero addresses (0x0000...0000)
- Pointing to actually deployed contracts

### Issue: Database Connection Failed
**Solution**: 
1. Ensure PostgreSQL is running
2. Check `DATABASE_URL` format
3. Verify database exists:
   ```bash
   createdb tokenization_platform
   ```

### Issue: Environment Variables Not Loading
**Solution**:
1. Check file permissions: `chmod 644 .env`
2. Verify file format (no BOM, Unix line endings)
3. Use absolute paths if needed
4. Try sourcing manually: `source .env`

## Security Notes

1. **Never commit `.env` files** to version control
2. **Change default JWT secrets** before production
3. **Use strong database passwords** in production
4. **Rotate private keys** regularly
5. **Use environment-specific configurations** for different deployments

## Development Workflow

1. **Initial setup:**
   ```bash
   git clone <repository>
   cd tokenization-backend
   cp .env.template .env
   source ./setup-env.sh
   ```

2. **Daily development:**
   ```bash
   ./check-env.sh          # Verify environment
   ./start-dev.sh          # Start with full checks
   # or
   source ./setup-env.sh   # Quick environment load
   cargo run               # Direct run
   ```

3. **After contract redeployment:**
   ```bash
   # Update contract addresses in .env
   ./check-env.sh          # Verify new addresses
   cargo run               # Test with new contracts
   ```

## Additional Configuration

For production deployments, additional environment variables may be required:

- **External APIs**: KYC providers, payment processors, oracles
- **Monitoring**: Logging, metrics, health checks
- **Storage**: File upload paths, cloud storage credentials  
- **Email**: SMTP configuration for notifications
- **Security**: Rate limiting, CORS, authentication providers

See `.env.template` for the complete list of available configuration options.

## Support

If you encounter issues:

1. Run `./check-env.sh` to identify configuration problems
2. Check the application logs for detailed error messages
3. Verify that all external services (database, blockchain node) are running
4. Ensure contract addresses match your deployed contracts

For contract deployment issues, see `README_DEPLOY.MD`.