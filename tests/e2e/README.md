# End-to-End Testing

This directory contains end-to-end tests for ConnectCare that test the full stack including MongoDB integration.

## Overview

The E2E tests:
- Start ConnectCare and MongoDB using Docker Compose
- Send real webhook requests with proper HMAC signatures
- Verify data is correctly filtered, mapped, and stored in MongoDB
- Test both happy paths and error scenarios

## Test Coverage

### Happy Path Tests
1. **Health Check** - Verify service is running
2. **Issue Created** - Test issue creation webhook with filtering
3. **Issue Updated** - Test issue update webhook
4. **Version Released** - Test version webhook with mapping transformation
5. **Project Created** - Test project webhook
6. **Issue Deleted** - Test delete operation
7. **MongoDB Persistence** - Verify data is stored correctly
8. **MongoDB Mapped Data** - Verify mapping transformations

### Error Handling Tests
1. **Invalid HMAC Signature** - Should return 401
2. **Missing Signature Header** - Should return 400
3. **Malformed JSON** - Should return 400
4. **Unsupported Event Type** - Should accept but filter out

## Running Tests Locally

### Prerequisites
- Docker and Docker Compose
- `curl`, `openssl`, and `mongosh` (MongoDB Shell)

### Quick Start

Run all E2E tests:
```bash
./run_e2e_tests.sh
```

Or manually:
```bash
# Start services
docker-compose -f docker-compose.test.yml up -d --build

# Wait for services
sleep 10

# Run tests
bash tests/e2e/run_tests.sh

# Cleanup
docker-compose -f docker-compose.test.yml down -v
```

### View Logs

```bash
# ConnectCare logs
docker-compose -f docker-compose.test.yml logs -f connectcare

# MongoDB logs
docker-compose -f docker-compose.test.yml logs -f mongodb
```

### Inspect MongoDB Data

```bash
# Connect to MongoDB
mongosh mongodb://localhost:27017/connectcare_test

# List all events
db.events.find().pretty()

# Count events
db.events.countDocuments()

# Find specific event
db.events.findOne({_eventType: "jira:issue_created"})
```

## Test Configuration

The tests use a specific configuration at `tests/e2e/config.test.json` which includes:

- MongoDB connection to the test container
- Two pipelines:
  1. Filter for issue events (created/updated) → store in MongoDB
  2. Filter for version releases → map fields → store in MongoDB

## CI/CD Integration

The tests are automatically run in GitHub Actions as part of the Rust workflow:

1. Unit tests run first
2. If unit tests pass, E2E tests run
3. Services are started with Docker Compose
4. Full test suite runs against live services
5. Logs are captured on failure
6. Services are cleaned up

## Adding New Tests

To add a new test:

1. Create a test function in `run_tests.sh`:
```bash
test_my_new_feature() {
    local payload='{"webhookEvent":"...","data":"..."}'
    local response=$(send_webhook "/jira/webhook" "$payload")
    local http_code=$(echo "$response" | tail -n1)
    [ "$http_code" = "200" ]
}
```

2. Add the test to the test runner:
```bash
run_test "My New Feature" test_my_new_feature
```

## Troubleshooting

### Services won't start

Check Docker logs:
```bash
docker-compose -f docker-compose.test.yml logs
```

### Tests fail to connect

Ensure services are healthy:
```bash
docker-compose -f docker-compose.test.yml ps
curl http://localhost:8080/-/healthz
```

### MongoDB connection issues

Verify MongoDB is accessible:
```bash
mongosh mongodb://localhost:27017/connectcare_test --eval "db.runCommand('ping')"
```

### Clean state between runs

Remove all containers and volumes:
```bash
docker-compose -f docker-compose.test.yml down -v
```

## Performance

Typical test run times:
- Service startup: ~10-15 seconds
- Test execution: ~5-10 seconds
- Total: ~20-30 seconds
