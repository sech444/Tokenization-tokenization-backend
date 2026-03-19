#!/bin/bash

# start-dev.sh - Development startup script for Tokenization Platform Backend
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "This script must be run from the tokenization-backend directory"
    exit 1
fi

print_status "Starting Tokenization Platform Backend in Development Mode..."

# Load environment variables from deployed.development.env
if [ -f "deployed.development.env" ]; then
    print_status "Loading environment variables from deployed.development.env..."
    export $(grep -v '^#' deployed.development.env | xargs)
    print_success "Environment variables loaded"
else
    print_warning "deployed.development.env not found, using default environment"
fi

# Set additional development defaults if not already set
export RUST_LOG=${RUST_LOG:-"debug,tokenization_platform=trace"}
export RUST_BACKTRACE=${RUST_BACKTRACE:-1}
export SERVER_HOST=${SERVER_HOST:-"127.0.0.1"}
export SERVER_PORT=${SERVER_PORT:-8080}

# Use DATABASE_URL from environment if set, otherwise default to local
if [ -z "$DATABASE_URL" ]; then
    export DATABASE_URL="postgresql://postgres:password@localhost:5432/tokenization_platform"
fi

# Check database connection based on URL type
print_status "Checking database connection..."
if [[ "$DATABASE_URL" == *"localhost"* || "$DATABASE_URL" == *"127.0.0.1"* ]]; then
    print_status "Detected local database, checking PostgreSQL service..."
    if ! pg_isready -h localhost -p 5432 -q; then
        print_warning "PostgreSQL doesn't seem to be running on localhost:5432"
        print_status "Attempting to start PostgreSQL service..."
        if command -v systemctl &> /dev/null; then
            sudo systemctl start postgresql || print_warning "Could not start PostgreSQL via systemctl"
        elif command -v brew &> /dev/null; then
            brew services start postgresql || print_warning "Could not start PostgreSQL via brew"
        else
            print_warning "Please start PostgreSQL manually"
        fi
    fi

    # Check if database exists, create if not (local only)
    print_status "Checking if local database exists..."
    if ! psql -lqt | cut -d \| -f 1 | grep -qw tokenization_platform; then
        print_status "Creating database 'tokenization_platform'..."
        createdb tokenization_platform || print_warning "Could not create database (may already exist)"
    fi
else
    print_status "Detected remote database connection"
    print_success "Using remote database: ${DATABASE_URL%%\?*}..." # Hide query params for security
fi

# Check if Redis is running (optional, with warning if not available)
if command -v redis-cli &> /dev/null; then
    if ! redis-cli ping > /dev/null 2>&1; then
        print_warning "Redis is not running. Some features may not work properly."
        print_status "To start Redis: redis-server"
    else
        print_success "Redis is running"
    fi
else
    print_warning "Redis CLI not found. Install Redis for full functionality."
fi

# Run database migrations
print_status "Running database migrations..."
if ! sqlx migrate run; then
    print_error "Database migrations failed"
    print_status "Make sure PostgreSQL is running and DATABASE_URL is correct"
    exit 1
fi
print_success "Database migrations completed"

# Check if Anvil (local blockchain) is running
print_status "Checking if local blockchain (Anvil) is running..."
if ! curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    http://127.0.0.1:8545 > /dev/null 2>&1; then
    print_warning "Local blockchain (Anvil) is not running on port 8545"
    print_status "To start Anvil: anvil --host 0.0.0.0 --port 8545"
    print_status "Or update BLOCKCHAIN_RPC_URL to point to your blockchain node"
fi

# Validate required contract addresses
print_status "Validating contract addresses..."
required_contracts=("CONTRACT_TOKEN_FACTORY" "CONTRACT_MARKETPLACE" "CONTRACT_COMPLIANCE" "CONTRACT_STAKING")
missing_contracts=()

for contract in "${required_contracts[@]}"; do
    if [ -z "${!contract}" ] || [ "${!contract}" == "0x0000000000000000000000000000000000000000" ]; then
        missing_contracts+=($contract)
    fi
done

if [ ${#missing_contracts[@]} -ne 0 ]; then
    print_error "Missing or invalid contract addresses:"
    printf '%s\n' "${missing_contracts[@]}"
    print_status "Please deploy contracts first or update deployed.development.env"
    exit 1
fi

print_success "All required contract addresses are configured"

# Display current configuration
print_status "Current configuration:"
echo "  Database URL: ${DATABASE_URL%%\?*}..." # Hide credentials
echo "  Server: ${SERVER_HOST}:${SERVER_PORT}"
echo "  Blockchain RPC: ${BLOCKCHAIN_RPC_URL:-http://127.0.0.1:8545}"
echo "  Environment: ${ENVIRONMENT:-development}"
echo "  Log Level: ${RUST_LOG}"

# Build the project
print_status "Building the project..."
if ! cargo build; then
    print_error "Build failed"
    exit 1
fi
print_success "Build completed"

# Start the server
print_success "Starting the Tokenization Platform Backend..."
print_status "Server will be available at http://${SERVER_HOST}:${SERVER_PORT}"
print_status "Health check: http://${SERVER_HOST}:${SERVER_PORT}/health"
print_status "Press Ctrl+C to stop the server"

echo ""
echo "========================================"
echo "🚀 TOKENIZATION PLATFORM BACKEND 🚀"
echo "========================================"
echo ""

# Run the application
exec cargo run
