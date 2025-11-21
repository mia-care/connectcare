#!/bin/bash

# Note: We don't use 'set -e' because we want to run all tests even if some fail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "================================================"
echo "Installing Dependencies"
echo "================================================"

# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "${BLUE}Detected Linux - installing dependencies...${NC}"
    
    # Update package list
    sudo apt-get update -qq
    
    # Install MongoDB shell
    if ! command -v mongosh &> /dev/null; then
        echo "Installing MongoDB Shell..."
        curl -fsSL https://www.mongodb.org/static/pgp/server-7.0.asc | sudo gpg --dearmor -o /usr/share/keyrings/mongodb-server-7.0.gpg
        echo "deb [ signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] https://repo.mongodb.org/apt/ubuntu $(lsb_release -cs)/mongodb-org/7.0 multiverse" | sudo tee /etc/apt/sources.list.d/mongodb-org-7.0.list
        sudo apt-get update -qq
        sudo apt-get install -y mongodb-mongosh
    fi
    
    # Install curl and openssl if not present
    sudo apt-get install -y curl openssl
    
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${BLUE}Detected macOS - checking dependencies...${NC}"
    
    # Install MongoDB shell via Homebrew
    if ! command -v mongosh &> /dev/null; then
        echo "Installing MongoDB Shell..."
        if command -v brew &> /dev/null; then
            brew install mongosh
        else
            echo -e "${YELLOW}Homebrew not found. Please install mongosh manually:${NC}"
            echo "brew install mongosh"
            exit 1
        fi
    fi
    
    # curl and openssl should be available by default on macOS
fi

echo -e "${GREEN}Dependencies installed successfully${NC}"
echo ""

# Configuration
BASE_URL="http://localhost:8080"
SECRET="test_secret_12345"
MONGO_URI="mongodb://localhost:27017"
DB_NAME="connectcare_test"
COLLECTION_NAME="events"

# Counter for tests
TESTS_PASSED=0
TESTS_FAILED=0

echo "================================================"
echo "ConnectCare E2E Tests"
echo "================================================"

# Function to generate HMAC signature
generate_signature() {
    local data="$1"
    echo -n "$data" | openssl dgst -sha256 -hmac "$SECRET" | cut -d' ' -f2
}

# Function to make a webhook request
send_webhook() {
    local path="$1"
    local data="$2"
    local signature=$(generate_signature "$data")
    
    # Uncomment for debugging:
    # echo "DEBUG: Sending to $BASE_URL$path" >&2
    # echo "DEBUG: Signature: sha256=$signature" >&2
    # echo "DEBUG: Payload: $data" >&2
    
    curl -s -w "\n%{http_code}" -X POST "$BASE_URL$path" \
        -H "Content-Type: application/json" \
        -H "X-Hub-Signature: sha256=$signature" \
        -d "$data"
}

# Function to make a request with wrong signature
send_webhook_wrong_signature() {
    local path="$1"
    local data="$2"
    
    curl -s -w "\n%{http_code}" -X POST "$BASE_URL$path" \
        -H "Content-Type: application/json" \
        -H "X-Hub-Signature: sha256=wrongsignature123" \
        -d "$data"
}

# Function to check MongoDB for a document
check_mongo_document() {
    local event_id="$1"
    mongosh "$MONGO_URI/$DB_NAME" --quiet --eval "db.$COLLECTION_NAME.findOne({_id: '$event_id'})" 2>/dev/null || echo ""
}

# Function to count documents in MongoDB
count_mongo_documents() {
    local result=$(mongosh "$MONGO_URI/$DB_NAME" --quiet --eval "db.$COLLECTION_NAME.countDocuments({})" 2>/dev/null || echo "")
    echo "${result:-0}"
}

# Function to run a test
run_test() {
    local test_name="$1"
    local test_func="$2"
    
    echo ""
    echo "Test: $test_name"
    
    # Capture both stdout and stderr, and the exit code
    local output
    local exit_code
    output=$($test_func 2>&1)
    exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✓ PASSED${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAILED${NC}"
        if [ -n "$output" ]; then
            echo -e "${YELLOW}Output: $output${NC}"
        fi
        ((TESTS_FAILED++))
    fi
}

# Wait for services to be ready
echo "Waiting for ConnectCare to be ready..."
for i in {1..30}; do
    if curl -s "$BASE_URL/-/healthz" > /dev/null 2>&1; then
        echo "ConnectCare is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}ConnectCare failed to start${NC}"
        exit 1
    fi
    sleep 1
done

# Clear MongoDB collection before tests
echo "Clearing MongoDB collection..."
mongosh "$MONGO_URI/$DB_NAME" --quiet --eval "db.$COLLECTION_NAME.deleteMany({})" > /dev/null 2>&1

echo ""
echo "Starting tests..."

# ================================================
# Test 1: Health check endpoint
# ================================================
test_health_check() {
    local response=$(curl -s -w "\n%{http_code}" "$BASE_URL/-/healthz")
    local http_code=$(echo "$response" | tail -n1)
    [ "$http_code" = "200" ]
}

# ================================================
# Test 2: Valid issue created webhook
# ================================================
test_issue_created() {
    local payload='{"webhookEvent":"jira:issue_created","timestamp":1234567890,"issue":{"id":"10001","key":"PROJ-123","fields":{"summary":"Test Issue","priority":"High"}}}'
    local response=$(send_webhook "/jira/webhook" "$payload")
    local http_code=$(echo "$response" | tail -n1)
    local body=$(echo "$response" | sed '$d')
    
    echo "DEBUG: HTTP Code: $http_code" >&2
    echo "DEBUG: Response Body: $body" >&2
    
    if [ "$http_code" != "200" ]; then
        echo "Expected 200, got $http_code. Response: $body" >&2
        return 1
    fi
    return 0
}

# ================================================
# Test 3: Valid issue updated webhook
# ================================================
test_issue_updated() {
    local payload='{"webhookEvent":"jira:issue_updated","timestamp":1234567890,"issue":{"id":"10002","key":"PROJ-124","fields":{"summary":"Updated Issue"}},"changelog":{"items":[{"field":"status","fromString":"Open","toString":"In Progress"}]}}'
    local response=$(send_webhook "/jira/webhook" "$payload")
    local http_code=$(echo "$response" | tail -n1)
    local body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" != "200" ]; then
        echo "Expected 200, got $http_code. Response: $body" >&2
        return 1
    fi
    return 0
}

# ================================================
# Test 4: Invalid signature
# ================================================
test_invalid_signature() {
    local payload='{"webhookEvent":"jira:issue_created","issue":{"id":"10003","key":"PROJ-125"}}'
    local response=$(send_webhook_wrong_signature "/jira/webhook" "$payload")
    local http_code=$(echo "$response" | tail -n1)
    [ "$http_code" = "401" ]
}

# ================================================
# Test 5: Missing signature header
# ================================================
test_missing_signature() {
    local payload='{"webhookEvent":"jira:issue_created","issue":{"id":"10004","key":"PROJ-126"}}'
    local response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/jira/webhook" \
        -H "Content-Type: application/json" \
        -d "$payload")
    local http_code=$(echo "$response" | tail -n1)
    [ "$http_code" = "400" ]
}

# ================================================
# Test 6: Version released with mapping
# ================================================
test_version_released() {
    local payload='{"webhookEvent":"jira:version_released","timestamp":1234567890,"version":{"id":"20001","name":"v1.0.0","projectId":"10000","released":true}}'
    local response=$(send_webhook "/jira/webhook" "$payload")
    local http_code=$(echo "$response" | tail -n1)
    [ "$http_code" = "200" ]
}

# ================================================
# Test 7: Unsupported event type (should accept but filter)
# ================================================
test_unsupported_event() {
    local payload='{"webhookEvent":"jira:board_created","board":{"id":"1","name":"Test Board"}}'
    local response=$(send_webhook "/jira/webhook" "$payload")
    local http_code=$(echo "$response" | tail -n1)
    local body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" != "200" ]; then
        echo "Expected 200, got $http_code. Response: $body" >&2
        return 1
    fi
    return 0
}

# ================================================
# Test 8: Malformed JSON
# ================================================
test_malformed_json() {
    local payload='{"webhookEvent":"jira:issue_created", INVALID JSON'
    local signature=$(generate_signature "$payload")
    local response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/jira/webhook" \
        -H "Content-Type: application/json" \
        -H "X-Hub-Signature: sha256=$signature" \
        -d "$payload")
    local http_code=$(echo "$response" | tail -n1)
    [ "$http_code" = "400" ]
}

# ================================================
# Test 9: Check MongoDB persistence (issue created)
# ================================================
test_mongo_persistence() {
    sleep 2  # Wait for async processing
    local count=$(count_mongo_documents)
    # Should have at least 2 documents (issue_created and issue_updated from filter)
    [ "$count" -ge 2 ]
}

# ================================================
# Test 10: Check mapped version data in MongoDB
# ================================================
test_mongo_mapped_data() {
    sleep 1
    local result=$(mongosh "$MONGO_URI/$DB_NAME" --quiet --eval "db.$COLLECTION_NAME.findOne({versionName: 'v1.0.0'})" 2>/dev/null || echo "")
    echo "$result" | grep -q "versionName" && echo "$result" | grep -q "v1.0.0"
}

# ================================================
# Test 11: Issue deleted webhook (delete operation)
# ================================================
test_issue_deleted() {
    # First create an issue
    local payload_create='{"webhookEvent":"jira:issue_created","timestamp":1234567890,"issue":{"id":"10099","key":"PROJ-999","fields":{"summary":"To Be Deleted"}}}'
    send_webhook "/jira/webhook" "$payload_create" > /dev/null
    sleep 1
    
    # Then delete it
    local payload_delete='{"webhookEvent":"jira:issue_deleted","timestamp":1234567891,"issue":{"id":"10099","key":"PROJ-999"}}'
    local response=$(send_webhook "/jira/webhook" "$payload_delete")
    local http_code=$(echo "$response" | tail -n1)
    [ "$http_code" = "200" ]
}

# ================================================
# Test 12: Project created webhook
# ================================================
test_project_created() {
    local payload='{"webhookEvent":"project_created","timestamp":1234567890,"project":{"id":"30001","key":"NEWPROJ","name":"New Project"}}'
    local response=$(send_webhook "/jira/webhook" "$payload")
    local http_code=$(echo "$response" | tail -n1)
    [ "$http_code" = "200" ]
}

# Run all tests
run_test "Health Check" test_health_check
run_test "Issue Created Webhook" test_issue_created
run_test "Issue Updated Webhook" test_issue_updated
run_test "Invalid HMAC Signature" test_invalid_signature
run_test "Missing Signature Header" test_missing_signature
run_test "Version Released with Mapping" test_version_released
run_test "Unsupported Event Type" test_unsupported_event
run_test "Malformed JSON Payload" test_malformed_json
run_test "MongoDB Persistence" test_mongo_persistence
run_test "MongoDB Mapped Data" test_mongo_mapped_data
run_test "Issue Deleted Webhook" test_issue_deleted
run_test "Project Created Webhook" test_project_created

# Summary
echo ""
echo "================================================"
echo "Test Summary"
echo "================================================"
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
echo "Total: $((TESTS_PASSED + TESTS_FAILED))"
echo "================================================"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
