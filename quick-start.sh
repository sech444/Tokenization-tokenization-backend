#!/bin/bash

# Tokenization Platform - Quick Start Script
# This script gets your development environment up and running in minutes

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Project info
PROJECT_NAME="Tokenization Platform"
VERSION="1.0.0"

# Banner
echo -e "${PURPLE}"
echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║                                                                      ║"
echo "║    🚀 TOKENIZATION PLATFORM - QUICK START                           ║"
echo "║                                                                      ║"
echo "║    Real Estate & Business Tokenization Platform                     ║"
echo "║    Built with Rust, PostgreSQL, and Blockchain Technology           ║"
echo "║                                                                      ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo ""

# Function to print status
print_status() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to wait for service
wait_for_service() {
    local service_name=$1
    local host=$2
    local port=$3
    local max_attempts=${4:-30}
    local attempt=0

    print_status "Waiting for $service_name to be ready..."

    while [ $attempt -lt $max_attempts ]; do
        if nc -z "$host" "$port" 2>/dev/null; then
            print_success "$service_name is ready!"
            return 0
        fi

        attempt=$((attempt + 1))
        echo -n "."
        sleep 2
    done

    print_error "$service_name failed to start within $((max_attempts * 2)) seconds"
    return 1
}

# Function to check Docker and Docker Compose
check_docker() {
    if ! command_exists docker; then
        print_error "Docker is not installed. Please install Docker first."
        print_info "Visit: https://docs.docker.com/get-docker/"
        exit 1
    fi

    if ! docker info >/dev/null 2>&1; then
        print_error "Docker daemon is not running. Please start Docker."
        exit 1
    fi

    if ! command_exists docker-compose && ! docker compose version >/dev/null 2>&1; then
        print_error "Docker Compose is not installed. Please install Docker Compose."
        exit 1
    fi

    print_success "Docker and Docker Compose are ready"
}

# Function to setup environment
setup_environment() {
    print_status "Setting up environment configuration..."

    # Copy environment file if .env doesn't exist
    if [[ ! -f ".env" ]]; then
        if [[ -f ".env.dev" ]]; then
            cp .env.dev .env
            print_success "Created .env from .env.dev"
        elif [[ -f ".env.example" ]]; then
            cp .env.example .env
            print_warning "Created .env from .env.example - you may need to edit it"
        else
            print_error ".env.example not found. Cannot create environment file."
            exit 1
        fi
    else
        print_info ".env already exists - using existing configuration"
    fi

    # Create necessary directories
    mkdir -p storage/documents
    mkdir -p logs
    print_success "Created storage directories"
}

# Function to start services
start_services() {
    print_status "Starting core services with Docker Compose..."

    # Stop any existing containers
    docker-compose down >/dev/null 2>&1 || true

    # Start core services
    print_status "Starting PostgreSQL, Redis, and Ganache..."
    docker-compose up -d postgres redis ganache

    # Wait for services to be ready
    wait_for_service "PostgreSQL" "localhost" "5432" 30
    wait_for_service "Redis" "localhost" "6379" 15
    wait_for_service "Ganache" "localhost" "8545" 20

    # Start supporting services
    print_status "Starting supporting services..."
    docker-compose up -d mailhog

    wait_for_service "MailHog" "localhost" "1025" 10

    print_success "All services are running!"
}

# Function to setup database
setup_database() {
    print_status "Setting up database..."

    # Check if sqlx-cli is installed
    if ! command_exists sqlx; then
        print_status "Installing sqlx-cli..."
        if command_exists cargo; then
            cargo install sqlx-cli --no-default-features --features postgres
            print_success "sqlx-cli installed"
        else
            print_error "Cargo not found. Please install Rust first."
            exit 1
        fi
    fi

    # Run database migrations
    print_status "Running database migrations..."
    export DATABASE_URL="postgresql://postgres:postgres123@localhost:5432/tokenization_db"

    # Create database if it doesn't exist
    sqlx database create 2>/dev/null || print_info "Database already exists"

    # Run migrations
    sqlx migrate run
    print_success "Database migrations completed"

    # Create test database
    export TEST_DATABASE_URL="postgresql://postgres:postgres123@localhost:5432/tokenization_test_db"
    sqlx database create --database-url "$TEST_DATABASE_URL" 2>/dev/null || print_info "Test database already exists"
    sqlx migrate run --database-url "$TEST_DATABASE_URL"
    print_success "Test database setup completed"
}

# Function to build the application
build_application() {
    print_status "Building Rust application..."

    # Check if Rust is installed
    if ! command_exists cargo; then
        print_error "Rust/Cargo not found. Please install Rust first."
        print_info "Visit: https://rustup.rs/"
        exit 1
    fi

    # Build the application
    cargo build
    print_success "Application built successfully"
}

# Function to seed database with initial data
seed_database() {
    print_status "Seeding database with initial data..."

    # We'll add a SQL script to insert initial data
    cat << 'EOF' | psql postgresql://postgres:postgres123@localhost:5432/tokenization_db
-- Insert default admin user if not exists
INSERT INTO users (
    id,
    email,
    password_hash,
    first_name,
    last_name,
    role,
    status,
    email_verified
) VALUES (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    'admin@tokenization.local',
    '$2b$04$Vg4mTpjmgL5yJKxGN7hgjuqI0s7Lj9OaOgGhA4QcWRGy.EkVaEvWW', -- password: admin123
    'Platform',
    'Administrator',
    'admin',
    'active',
    true
) ON CONFLICT (email) DO NOTHING;

-- Insert sample project types
INSERT INTO projects (
    id,
    name,
    description,
    project_type,
    status,
    owner_id,
    location,
    total_value,
    minimum_investment,
    investment_period_months,
    property_details
) VALUES (
    'b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a12',
    'Sydney CBD Commercial Complex',
    'Premium commercial development in Sydney CBD with high rental yields',
    'commercial',
    'active',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    'Sydney, NSW, Australia',
    50000000,
    10000,
    24,
    '{"property_type": "commercial", "floors": 25, "units": 200}'
) ON CONFLICT (id) DO NOTHING;

-- Insert compliance profile for admin user
INSERT INTO compliance_profiles (
    id,
    user_id,
    risk_rating,
    investor_type,
    accredited_investor,
    accreditation_verified
) VALUES (
    'c2eebc99-9c0b-4ef8-bb6d-6bb9bd380a13',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    'low',
    'institutional',
    true,
    true
) ON CONFLICT (user_id) DO NOTHING;

EOF

    print_success "Database seeded with initial data"
}

# Function to display connection info
display_connection_info() {
    echo ""
    echo -e "${PURPLE}╔══════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${PURPLE}║                       🎉 SETUP COMPLETE!                            ║${NC}"
    echo -e "${PURPLE}╚══════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}Your Tokenization Platform is ready! Here are your connection details:${NC}"
    echo ""
    echo -e "${CYAN}🌐 Application Services:${NC}"
    echo -e "   • API Server: ${YELLOW}http://localhost:8080${NC} (will start with 'cargo run')"
    echo -e "   • Health Check: ${YELLOW}http://localhost:8080/health${NC}"
    echo -e "   • API Documentation: ${YELLOW}http://localhost:8080/docs${NC}"
    echo ""
    echo -e "${CYAN}🗄️  Database Services:${NC}"
    echo -e "   • PostgreSQL: ${YELLOW}localhost:5432${NC}"
    echo -e "   • Database: ${YELLOW}tokenization_db${NC}"
    echo -e "   • Username: ${YELLOW}postgres${NC}"
    echo -e "   • Password: ${YELLOW}postgres123${NC}"
    echo ""
    echo -e "${CYAN}🧪 Development Tools:${NC}"
    echo -e "   • MailHog (Email Testing): ${YELLOW}http://localhost:8025${NC}"
    echo -e "   • Redis: ${YELLOW}localhost:6379${NC}"
    echo -e "   • Ganache (Blockchain): ${YELLOW}http://localhost:8545${NC}"
    echo ""
    echo -e "${CYAN}👤 Default Admin Account:${NC}"
    echo -e "   • Email: ${YELLOW}admin@tokenization.local${NC}"
    echo -e "   • Password: ${YELLOW}admin123${NC}"
    echo ""
    echo -e "${GREEN}🚀 Next Steps:${NC}"
    echo -e "   1. Start the application: ${BLUE}cargo run${NC}"
    echo -e "   2. Open your browser: ${BLUE}http://localhost:8080${NC}"
    echo -e "   3. Login with the admin account above"
    echo -e "   4. Start tokenizing your real estate projects!"
    echo ""
    echo -e "${GREEN}📚 Development Commands:${NC}"
    echo -e "   • Run with hot reload: ${BLUE}cargo watch -x run${NC}"
    echo -e "   • Run tests: ${BLUE}cargo test${NC}"
    echo -e "   • View logs: ${BLUE}docker-compose logs -f${NC}"
    echo -e "   • Stop services: ${BLUE}docker-compose down${NC}"
    echo ""
    echo -e "${GREEN}💡 Useful URLs:${NC}"
    echo -e "   • Email Testing: ${YELLOW}http://localhost:8025${NC}"
    echo -e "   • Database Admin: Install pgAdmin and connect to localhost:5432"
    echo ""
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."

    local missing_deps=0

    # Check essential tools
    if ! command_exists docker; then
        print_error "Docker not found"
        missing_deps=$((missing_deps + 1))
    fi

    if ! command_exists cargo; then
        print_error "Rust/Cargo not found"
        missing_deps=$((missing_deps + 1))
    fi

    if ! command_exists psql; then
        print_warning "PostgreSQL client (psql) not found - will use Docker version"
    fi

    if [[ $missing_deps -gt 0 ]]; then
        print_error "Missing $missing_deps critical dependencies"
        print_info "Run ./setup-check.sh for detailed installation instructions"
        exit 1
    fi

    print_success "All prerequisites check passed"
}

# Main execution flow
main() {
    print_status "Starting quick setup for $PROJECT_NAME v$VERSION"
    echo ""

    # Step 1: Check prerequisites
    check_prerequisites

    # Step 2: Setup environment
    setup_environment

    # Step 3: Check and start Docker services
    check_docker
    start_services

    # Step 4: Setup database
    setup_database

    # Step 5: Build application
    build_application

    # Step 6: Seed database
    seed_database

    # Step 7: Display connection information
    display_connection_info

    print_success "Quick start completed successfully!"
    echo ""
    echo -e "${GREEN}Run the following command to start your application:${NC}"
    echo -e "${BLUE}cargo run${NC}"
    echo ""
}

# Error handling
trap 'print_error "Setup failed. Check the error messages above."' ERR

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --check        Run setup check only"
        echo "  --clean        Clean and restart everything"
        echo ""
        echo "This script sets up the complete development environment for"
        echo "the Tokenization Platform including Docker services, database,"
        echo "and initial data seeding."
        exit 0
        ;;
    --check)
        if [[ -f "setup-check.sh" ]]; then
            ./setup-check.sh
        else
            check_prerequisites
        fi
        exit 0
        ;;
    --clean)
        print_status "Cleaning existing setup..."
        docker-compose down -v 2>/dev/null || true
        docker system prune -f 2>/dev/null || true
        rm -f .env
        print_success "Cleanup completed"
        ;;
esac

# Run main function
main "$@"
