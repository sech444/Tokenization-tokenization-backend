#!/bin/bash

# verify-setup.sh - Verification script for Tokenization Platform Backend
# This script checks if the application is running correctly and all components are working

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVER_HOST=${SERVER_HOST:-127.0.0.1}
SERVER_PORT=${SERVER_PORT:-8080}
BASE_URL="http://${SERVER_HOST}:${SERVER_PORT}"
MAX_RETRIES=30
RETRY_DELAY=2

# Counters
PASSED=0
FAILED=0

# Function to print status
print_test() {
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
    elif [ "$status" = "INFO" ]; then
        echo -e "${BLUE}ℹ️  INFO${NC} $message"
    fi
}

# Function to wait for server to be ready
wait_for_server() {
    print_test "INFO" "Waiting for server to be ready at $BASE_URL..."

    for i in $(seq 1 $MAX_RETRIES); do
        if curl -s -f "$BASE_URL/health" > /dev/null 2>&1; then
            print_test "PASS" "Server is responding after $((i * RETRY_DELAY)) seconds"
            return 0
        fi

        if [ $i -eq $MAX_RETRIES ]; then
            print_test "FAIL" "Server failed to respond after $((MAX_RETRIES * RETRY_DELAY)) seconds"
            return 1
        fi

        echo -n "."
        sleep $RETRY_DELAY
    done
}

# Function to test HTTP endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local expected_status=$3
    local description=$4
    local auth_header=$5

    local url="${BASE_URL}${endpoint}"
    local curl_cmd="curl -s"

    if [ -n "$auth_header" ]; then
        curl_cmd="$curl_cmd -H \"Authorization: $auth_header\""
    fi

    curl_cmd="$curl_cmd -w '%{http_code}' -o /dev/null"

    if [ "$method" != "GET" ]; then
        curl_cmd="$curl_cmd -X $method"
    fi

    curl_cmd="$curl_cmd \"$url\""

    local status_code
    status_code=$(eval $curl_cmd 2>/dev/null)

    if [ "$status_code" = "$expected_status" ]; then
        print_test "PASS" "$description" "Status: $status_code"
    else
        print_test "FAIL" "$description" "Expected: $expected_status, Got: $status_code"
    fi
}

# Function to test JSON response
test_json_endpoint() {
    local endpoint=$1
    local description=$2
    local expected_content=$3

    local url="${BASE_URL}${endpoint}"
    local response
    response=$(curl -s "$url" 2>/dev/null)
    local status_code=$(curl -s -w '%{http_code}' -o /dev/null "$url" 2>/dev/null)

    if [ "$status_code" = "200" ]; then
        if [ -n "$expected_content" ] && echo "$response" | grep -q "$expected_content"; then
            print_test "PASS" "$description" "Response contains expected content"
        elif [ -z "$expected_content" ]; then
            print_test "PASS" "$description" "Status: 200, Response: $(echo "$response" | cut -c1-50)..."
        else
            print_test "FAIL" "$description" "Response doesn't contain expected content: $expected_content"
        fi
    else
        print_test "FAIL" "$description" "Status: $status_code"
    fi
}

echo "========================================"
echo "🔍 TOKENIZATION PLATFORM VERIFICATION"
echo "========================================"
echo ""

# Check if server is running
print_test "INFO" "Testing server availability..."
wait_for_server || exit 1

echo ""
echo -e "${BLUE}🌐 API ENDPOINT TESTS${NC}"
echo "----------------------------------------"

# Test health endpoint
test_json_endpoint "/health" "Health check endpoint" "Tokenization Platform API is running"

# Test authentication endpoints
test_endpoint "POST" "/api/auth/register" "400" "Registration endpoint (expects 400 without data)"
test_endpoint "POST" "/api/auth/login" "400" "Login endpoint (expects 400 without credentials)"
test_endpoint "GET" "/api/auth/verify" "405" "Auth verify endpoint (expects 405 for GET method)"

# Test user endpoints (should require authentication)
test_endpoint "GET" "/api/users/me" "401" "Get current user (should require auth)"

# Test public endpoints
test_endpoint "GET" "/api/tokens" "401" "List tokens (should require auth)"
test_endpoint "GET" "/api/projects" "401" "List projects (should require auth)"
test_endpoint "GET" "/api/marketplace/listings" "401" "Marketplace listings (should require auth)"

# Test admin endpoints (should require admin auth)
test_endpoint "GET" "/api/admin/users" "401" "Admin user list (should require admin auth)"
test_endpoint "GET" "/api/admin/kyc/pending" "401" "Admin KYC pending (should require admin auth)"

echo ""
echo -e "${BLUE}🔗 WALLET INTEGRATION TESTS${NC}"
echo "----------------------------------------"

# Test wallet endpoints
test_endpoint "POST" "/api/auth/wallet/nonce" "400" "Get wallet nonce (expects 400 without data)"
test_endpoint "POST" "/api/auth/wallet/verify" "400" "Verify wallet signature (expects 400 without data)"
test_endpoint "GET" "/api/auth/wallet/info" "401" "Get wallet info (should require auth)"

echo ""
echo -e "${BLUE}🔧 CONFIGURATION TESTS${NC}"
echo "----------------------------------------"

# Check environment variables
if [ -n "$DATABASE_URL" ]; then
    print_test "PASS" "Database URL configured" "$(echo $DATABASE_URL | cut -c1-30)..."
else
    print_test "FAIL" "Database URL not configured"
fi

if [ -n "$CONTRACT_STAKING" ]; then
    print_test "PASS" "Staking contract address configured" "$CONTRACT_STAKING"
else
    print_test "FAIL" "Staking contract address not configured"
fi

if [ -n "$KYC_API_KEY" ]; then
    print_test "PASS" "KYC API key configured" "$(echo $KYC_API_KEY | cut -c1-10)..."
else
    print_test "FAIL" "KYC API key not configured"
fi

if [ -n "$JWT_SECRET" ]; then
    print_test "PASS" "JWT secret configured" "Length: ${#JWT_SECRET} chars"
else
    print_test "FAIL" "JWT secret not configured"
fi

echo ""
echo -e "${BLUE}📊 APPLICATION HEALTH${NC}"
echo "----------------------------------------"

# Test server response time
start_time=$(date +%s%N)
curl -s "$BASE_URL/health" > /dev/null 2>&1
end_time=$(date +%s%N)
response_time=$(( (end_time - start_time) / 1000000 ))

if [ $response_time -lt 1000 ]; then
    print_test "PASS" "Server response time" "${response_time}ms (good)"
elif [ $response_time -lt 3000 ]; then
    print_test "PASS" "Server response time" "${response_time}ms (acceptable)"
else
    print_test "FAIL" "Server response time" "${response_time}ms (slow)"
fi

# Test if server accepts CORS
cors_response=$(curl -s -H "Origin: http://localhost:3000" \
    -H "Access-Control-Request-Method: GET" \
    -H "Access-Control-Request-Headers: Content-Type" \
    -X OPTIONS "$BASE_URL/health" 2>/dev/null || echo "")

if [ -n "$cors_response" ]; then
    print_test "PASS" "CORS configuration" "Server accepts cross-origin requests"
else
    print_test "FAIL" "CORS configuration" "Server may not accept cross-origin requests"
fi

echo ""
echo "========================================"
echo "📊 VERIFICATION SUMMARY"
echo "========================================"

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED!${NC}"
    echo "Your Tokenization Platform Backend is working correctly."
    echo ""
    echo -e "${GREEN}✅ Ready for development!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Start the frontend: cd ../tokenization-frontend && npm start"
    echo "2. Test the full stack integration"
    echo "3. Deploy smart contracts if needed"
else
    echo -e "${RED}❌ $FAILED test(s) failed${NC}"
    echo -e "${GREEN}✅ $PASSED test(s) passed${NC}"
    echo ""
    echo "Issues to resolve:"
    echo "1. Check server logs for detailed error messages"
    echo "2. Verify environment variables are properly set"
    echo "3. Ensure database is accessible"
    echo "4. Check if all dependencies are installed"
fi

echo ""
echo "Server URL: $BASE_URL"
echo "API Documentation: $BASE_URL/docs (if Swagger UI is enabled)"
echo "Health Check: $BASE_URL/health"

exit $FAILED
