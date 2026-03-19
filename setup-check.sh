#!/bin/bash

# Tokenization Platform - Setup Verification Script
# This script checks if all required dependencies are installed

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Tracking variables
MISSING_DEPS=0
WARNINGS=0

echo -e "${BLUE}🚀 Tokenization Platform - Setup Verification${NC}"
echo "=============================================="
echo ""

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check version
check_version() {
    local cmd="$1"
    local version_cmd="$2"
    local min_version="$3"
    local name="$4"

    if command_exists "$cmd"; then
        local current_version=$($version_cmd 2>/dev/null | head -n1)
        echo -e "${GREEN}✅ $name: $current_version${NC}"
        return 0
    else
        echo -e "${RED}❌ $name: Not installed${NC}"
        return 1
    fi
}

# Function to print installation instructions
print_install() {
    local name="$1"
    local instructions="$2"
    echo -e "${YELLOW}📦 To install $name:${NC}"
    echo -e "   $instructions"
    echo ""
}

echo -e "${BLUE}Checking Core Dependencies...${NC}"
echo "--------------------------------"

# Check Rust
if check_version "rustc" "rustc --version" "1.70" "Rust"; then
    CARGO_VERSION=$(cargo --version 2>/dev/null)
    echo -e "${GREEN}✅ Cargo: $CARGO_VERSION${NC}"
else
    echo -e "${RED}❌ Rust: Not installed${NC}"
    print_install "Rust" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    MISSING_DEPS=$((MISSING_DEPS + 1))
fi

# Check PostgreSQL
if command_exists "psql"; then
    PG_VERSION=$(psql --version 2>/dev/null)
    echo -e "${GREEN}✅ PostgreSQL: $PG_VERSION${NC}"

    # Check if PostgreSQL service is running
    if pgrep -x "postgres" > /dev/null || pgrep -f "postgresql" > /dev/null; then
        echo -e "${GREEN}✅ PostgreSQL Service: Running${NC}"
    else
        echo -e "${YELLOW}⚠️  PostgreSQL Service: Not running${NC}"
        echo -e "   Start with: ${BLUE}sudo systemctl start postgresql${NC} or ${BLUE}brew services start postgresql${NC}"
        WARNINGS=$((WARNINGS + 1))
    fi
else
    echo -e "${RED}❌ PostgreSQL: Not installed${NC}"
    print_install "PostgreSQL" "
    Ubuntu/Debian: sudo apt install postgresql postgresql-contrib
    macOS: brew install postgresql
    Windows: Download from https://www.postgresql.org/download/"
    MISSING_DEPS=$((MISSING_DEPS + 1))
fi

# Check Docker
if check_version "docker" "docker --version" "20.0" "Docker"; then
    # Check if Docker daemon is running
    if docker info >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Docker Daemon: Running${NC}"
    else
        echo -e "${YELLOW}⚠️  Docker Daemon: Not running${NC}"
        echo -e "   Start Docker Desktop or run: ${BLUE}sudo systemctl start docker${NC}"
        WARNINGS=$((WARNINGS + 1))
    fi
else
    echo -e "${RED}❌ Docker: Not installed${NC}"
    print_install "Docker" "
    Ubuntu: curl -fsSL https://get.docker.com -o get-docker.sh && sh get-docker.sh
    macOS/Windows: Download Docker Desktop from https://www.docker.com/products/docker-desktop"
    MISSING_DEPS=$((MISSING_DEPS + 1))
fi

# Check Docker Compose
if command_exists "docker-compose" || command_exists "docker compose"; then
    if command_exists "docker-compose"; then
        COMPOSE_VERSION=$(docker-compose --version 2>/dev/null)
        echo -e "${GREEN}✅ Docker Compose: $COMPOSE_VERSION${NC}"
    else
        COMPOSE_VERSION=$(docker compose version 2>/dev/null)
        echo -e "${GREEN}✅ Docker Compose: $COMPOSE_VERSION${NC}"
    fi
else
    echo -e "${RED}❌ Docker Compose: Not installed${NC}"
    print_install "Docker Compose" "
    Usually included with Docker Desktop
    Linux: sudo curl -L \"https://github.com/docker/compose/releases/download/v2.23.0/docker-compose-\$(uname -s)-\$(uname -m)\" -o /usr/local/bin/docker-compose && sudo chmod +x /usr/local/bin/docker-compose"
    MISSING_DEPS=$((MISSING_DEPS + 1))
fi

echo ""
echo -e "${BLUE}Checking Additional Tools...${NC}"
echo "----------------------------"

# Check Node.js (for smart contracts)
if check_version "node" "node --version" "16.0" "Node.js"; then
    NPM_VERSION=$(npm --version 2>/dev/null)
    echo -e "${GREEN}✅ NPM: v$NPM_VERSION${NC}"
else
    echo -e "${RED}❌ Node.js: Not installed${NC}"
    print_install "Node.js" "
    Visit https://nodejs.org/ or use:
    Ubuntu: curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash - && sudo apt-get install -y nodejs
    macOS: brew install node"
    MISSING_DEPS=$((MISSING_DEPS + 1))
fi

# Check Git
if check_version "git" "git --version" "2.0" "Git"; then
    :
else
    echo -e "${RED}❌ Git: Not installed${NC}"
    print_install "Git" "
    Ubuntu/Debian: sudo apt install git
    macOS: xcode-select --install
    Windows: Download from https://git-scm.com/"
    MISSING_DEPS=$((MISSING_DEPS + 1))
fi

# Check curl
if command_exists "curl"; then
    echo -e "${GREEN}✅ curl: Available${NC}"
else
    echo -e "${YELLOW}⚠️  curl: Not installed${NC}"
    echo -e "   Install with: ${BLUE}sudo apt install curl${NC} or ${BLUE}brew install curl${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

echo ""
echo -e "${BLUE}Checking Rust Tools...${NC}"
echo "---------------------"

if command_exists "cargo"; then
    # Check for sqlx-cli
    if cargo install --list | grep -q "sqlx-cli"; then
        echo -e "${GREEN}✅ sqlx-cli: Installed${NC}"
    else
        echo -e "${YELLOW}⚠️  sqlx-cli: Not installed${NC}"
        echo -e "   Install with: ${BLUE}cargo install sqlx-cli --no-default-features --features postgres${NC}"
        WARNINGS=$((WARNINGS + 1))
    fi

    # Check for cargo-watch
    if cargo install --list | grep -q "cargo-watch"; then
        echo -e "${GREEN}✅ cargo-watch: Installed${NC}"
    else
        echo -e "${YELLOW}⚠️  cargo-watch: Not installed (optional for development)${NC}"
        echo -e "   Install with: ${BLUE}cargo install cargo-watch${NC}"
        WARNINGS=$((WARNINGS + 1))
    fi
fi

echo ""
echo -e "${BLUE}System Information...${NC}"
echo "--------------------"

# Check OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "${GREEN}✅ OS: Linux${NC}"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${GREEN}✅ OS: macOS${NC}"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    echo -e "${GREEN}✅ OS: Windows${NC}"
else
    echo -e "${YELLOW}⚠️  OS: $OSTYPE (may need additional setup)${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# Check available memory
if command_exists "free"; then
    MEMORY=$(free -h | awk '/^Mem:/ {print $2}')
    echo -e "${GREEN}✅ Memory: $MEMORY${NC}"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    MEMORY=$(sysctl -n hw.memsize | awk '{print int($1/1024/1024/1024) "GB"}')
    echo -e "${GREEN}✅ Memory: $MEMORY${NC}"
fi

# Check available disk space
if command_exists "df"; then
    DISK_SPACE=$(df -h . | awk 'NR==2 {print $4}')
    echo -e "${GREEN}✅ Available Disk Space: $DISK_SPACE${NC}"
fi

echo ""
echo -e "${BLUE}Project Structure Check...${NC}"
echo "-------------------------"

# Check if we're in the right directory
if [[ -f "Cargo.toml" ]]; then
    echo -e "${GREEN}✅ Cargo.toml found${NC}"
else
    echo -e "${YELLOW}⚠️  Cargo.toml not found - make sure you're in the project directory${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

if [[ -f "docker-compose.yml" ]]; then
    echo -e "${GREEN}✅ docker-compose.yml found${NC}"
else
    echo -e "${YELLOW}⚠️  docker-compose.yml not found${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

if [[ -f ".env.example" ]]; then
    echo -e "${GREEN}✅ .env.example found${NC}"
else
    echo -e "${YELLOW}⚠️  .env.example not found${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

if [[ -d "migrations" ]]; then
    MIGRATION_COUNT=$(ls migrations/*.sql 2>/dev/null | wc -l)
    echo -e "${GREEN}✅ Migrations directory found ($MIGRATION_COUNT files)${NC}"
else
    echo -e "${YELLOW}⚠️  Migrations directory not found${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

echo ""
echo -e "${BLUE}Network Connectivity...${NC}"
echo "----------------------"

# Check internet connectivity
if curl -s --connect-timeout 5 https://google.com > /dev/null; then
    echo -e "${GREEN}✅ Internet connectivity: Available${NC}"
else
    echo -e "${RED}❌ Internet connectivity: Failed${NC}"
    echo -e "   Required for downloading dependencies"
    MISSING_DEPS=$((MISSING_DEPS + 1))
fi

# Check if common ports are available
check_port() {
    local port=$1
    local service=$2

    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Port $port: In use (may conflict with $service)${NC}"
        WARNINGS=$((WARNINGS + 1))
    else
        echo -e "${GREEN}✅ Port $port: Available for $service${NC}"
    fi
}

check_port 5432 "PostgreSQL"
check_port 6379 "Redis"
check_port 8080 "Backend API"
check_port 8545 "Ganache"

echo ""
echo "=============================================="
echo -e "${BLUE}📊 Setup Summary${NC}"
echo "=============================================="

if [[ $MISSING_DEPS -eq 0 ]]; then
    echo -e "${GREEN}🎉 All required dependencies are installed!${NC}"
else
    echo -e "${RED}❌ Missing $MISSING_DEPS critical dependencies${NC}"
    echo -e "${RED}   Please install the missing dependencies before proceeding${NC}"
fi

if [[ $WARNINGS -gt 0 ]]; then
    echo -e "${YELLOW}⚠️  $WARNINGS warnings found${NC}"
    echo -e "${YELLOW}   Review the warnings above - some may affect development${NC}"
fi

echo ""
echo -e "${BLUE}📋 Next Steps:${NC}"

if [[ $MISSING_DEPS -eq 0 ]]; then
    echo -e "${GREEN}1. Copy environment file: ${BLUE}cp .env.example .env${NC}"
    echo -e "${GREEN}2. Edit .env with your settings: ${BLUE}nano .env${NC}"
    echo -e "${GREEN}3. Start development environment: ${BLUE}docker-compose up -d${NC}"
    echo -e "${GREEN}4. Run database migrations: ${BLUE}sqlx migrate run${NC}"
    echo -e "${GREEN}5. Start the application: ${BLUE}cargo run${NC}"
    echo ""
    echo -e "${GREEN}🚀 You're ready to start development!${NC}"
else
    echo -e "${RED}1. Install missing dependencies listed above${NC}"
    echo -e "${RED}2. Run this script again to verify installation${NC}"
    echo -e "${RED}3. Once all dependencies are installed, proceed with setup${NC}"
fi

echo ""
echo -e "${BLUE}For help, visit: https://github.com/your-org/tokenization-backend${NC}"
echo "=============================================="

# Exit with error code if dependencies are missing
if [[ $MISSING_DEPS -gt 0 ]]; then
    exit 1
fi

exit 0
