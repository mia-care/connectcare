#!/bin/bash

set -e

echo "Starting E2E tests..."

# Start services
echo "Starting services with docker-compose..."
docker-compose -f docker-compose.test.yml up -d --build

# Wait for services to be fully ready
echo "Waiting for services to be ready..."
sleep 10

# Run tests
echo "Running test suite..."
docker-compose -f docker-compose.test.yml exec -T connectcare /bin/sh -c "
    apk add --no-cache curl mongodb-tools bash openssl && \
    bash /app/tests/e2e/run_tests.sh
" || {
    # If exec doesn't work (distroless), run tests from host
    bash tests/e2e/run_tests.sh
}

TEST_EXIT_CODE=$?

# Cleanup
echo "Stopping services..."
docker-compose -f docker-compose.test.yml down -v

exit $TEST_EXIT_CODE
