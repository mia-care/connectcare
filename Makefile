.PHONY: help build test e2e clean docker docker-test lint fmt check run

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build the project
	cargo build --release

test: ## Run unit tests
	cargo test --verbose

e2e: ## Run end-to-end tests
	./run_e2e_tests.sh

e2e-start: ## Start E2E test environment (without running tests)
	docker-compose -f docker-compose.test.yml up -d --build
	@echo "Waiting for services..."
	@sleep 10
	@echo "Services ready!"
	@echo "ConnectCare: http://localhost:8080"
	@echo "MongoDB: mongodb://localhost:27017/connectcare_test"

e2e-stop: ## Stop E2E test environment
	docker-compose -f docker-compose.test.yml down -v

e2e-logs: ## Show E2E service logs
	docker-compose -f docker-compose.test.yml logs -f

check: ## Run cargo check
	cargo check

lint: ## Run clippy linter
	cargo clippy -- -D warnings

fmt: ## Format code with rustfmt
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt -- --check

clean: ## Clean build artifacts
	cargo clean
	docker-compose -f docker-compose.test.yml down -v 2>/dev/null || true
	docker-compose down -v 2>/dev/null || true

docker: ## Build Docker image
	docker build -t connectcare:latest .

docker-run: ## Run Docker container
	docker run -d \
		--name connectcare \
		-p 8080:8080 \
		-e JIRA_WEBHOOK_SECRET="test_secret" \
		-e RUST_LOG=info \
		-v $$(pwd)/config/config.json:/app/config/config.json:ro \
		connectcare:latest

docker-stop: ## Stop and remove Docker container
	docker stop connectcare 2>/dev/null || true
	docker rm connectcare 2>/dev/null || true

docker-compose-up: ## Start full stack with docker-compose
	docker-compose up -d

docker-compose-down: ## Stop full stack
	docker-compose down -v

run: ## Run the application locally
	cargo run --release

dev: ## Run the application in development mode
	RUST_LOG=debug cargo run

all: fmt lint test build ## Run all checks and build
