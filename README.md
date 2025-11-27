# ConnectCare

A high-performance webhook receiver service for Jira integrations with configurable event processing pipelines and MongoDB sink support, written in Rust.

## Features

- Jira webhook integration with HMAC-SHA256 signature validation
- Support for 15+ Jira event types (issues, projects, versions, issue links)
- Secure secret management (environment variables, files, or plain text)
- Event normalization and pipeline processing
- **CEL (Common Expression Language) filters** for event filtering
- **Handlebars template-based event mapping**
- **MongoDB sink** for persisted event storage
- Extensible processor and sink architecture
- Health check endpoints
- Structured logging with tracing
- **Docker & Docker Compose support** with distroless images

## Documentation

- [Quick Start](#quick-start) - Get started quickly with Docker or from source
- [DOCKER.md](DOCKER.md) - Comprehensive Docker deployment guide
- [API Endpoints](#api-endpoints) - Available HTTP endpoints
- [Pipeline Processing](#pipeline-processing) - Configure filters, mappers, and sinks
- [Configuration](#configuration) - Detailed configuration options

## Quick Start

### Prerequisites

#### Local Development
- Rust 1.70+ 
- Cargo
- MongoDB (if using database sink)

#### Docker
- Docker 20.10+
- Docker Compose 2.0+ (optional, for full stack)

### Installation

#### Option 1: Docker (Recommended)

**Build and run with Docker:**
```bash
# Build the image
docker build -t connectcare:latest .

# Run with environment variables
docker run -d \
  -p 8080:8080 \
  -e JIRA_WEBHOOK_SECRET="your-secret-here" \
  -e MONGO_URL="your-mongodb-connection-string" \
  -e RUST_LOG=info \
  -v $(pwd)/config/config.json:/app/config/config.json:ro \
  connectcare:latest
```

**Or use Docker Compose (includes MongoDB):**
```bash
# Set your secret
export JIRA_WEBHOOK_SECRET="your-secret-here"

# Start all services
docker-compose up -d

# View logs
docker-compose logs -f connectcare

# Stop services
docker-compose down
```

#### Option 2: Build from Source

```bash
cargo build --release
```

### Configuration

1. Copy the example configuration:
```bash
cp config/config.example.json config/config.json
```

2. Set your Jira webhook secret:
```bash
export JIRA_WEBHOOK_SECRET="your-secret-here"
```

3. Set MongoDB connection URL:
```bash
export MONGO_URL="mongodb://localhost:27017/connectcare/events"
```

4. Update `config/config.json` as needed.

### Running

```bash
# Set configuration path (optional, defaults to config/config.json)
export CONFIGURATION_PATH=config/config.json

# Set log level (optional)
export RUST_LOG=info

# Run the service
cargo run --release
```

The service will start on port 8080 (configurable).

## API Endpoints

### Health Checks

- `GET /-/healthz` - Health check endpoint
- `GET /-/ready` - Readiness check endpoint

### Jira Webhook

- `POST /jira/webhook` - Receives Jira webhook events (path configurable)

## Supported Jira Events

### Issue Events
- `jira:issue_created` (Write)
- `jira:issue_updated` (Write)
- `jira:issue_deleted` (Delete)

### Issue Link Events
- `issuelink_created` (Write)
- `issuelink_deleted` (Delete)

### Project Events
- `project_created` (Write)
- `project_updated` (Write)
- `project_deleted` (Delete)
- `project_soft_deleted` (Delete)
- `project_restored_deleted` (Write)

### Version Events
- `jira:version_created` (Write)
- `jira:version_updated` (Write)
- `jira:version_released` (Write)
- `jira:version_unreleased` (Write)
- `jira:version_deleted` (Delete)

## Configuration

### Secret Sources

Secrets can be loaded from three sources:

1. **Environment Variable:**
```json
{
  "secret": {
    "fromEnv": "JIRA_WEBHOOK_SECRET"
  }
}
```

2. **File:**
```json
{
  "secret": {
    "fromFile": "/path/to/secret.txt"
  }
}
```

3. **Plain Text** (not recommended for production):
```json
{
  "secret": "my-secret"
}
```

### Example Configuration

```json
{
  "server": {
    "port": 8080
  },
  "integrations": [
    {
      "source": {
        "type": "jira",
        "webhookPath": "/jira/webhook",
        "authentication": {
          "secret": {
            "fromEnv": "JIRA_WEBHOOK_SECRET"
          },
          "headerName": "X-Hub-Signature"
        }
      },
      "pipelines": [
        {
          "processors": [
            {
              "type": "filter",
              "celExpression": "eventType == 'jira:version_updated'"
            },
            {
              "type": "mapper",
              "outputEvent": {
                "deploymentName": "{{ version.name }}",
                "projectKey": "{{ version.projectId }}",
                "status": "{{ version.released }}",
                "timestamp": "{{ timestamp }}"
              }
            }
          ],
          "sinks": [
            {
              "type": "database",
              "provider": "MONGO"
            }
          ]
        }
      ]
    }
  ]
}
```

## Pipeline Processing

### Overview

Events flow through configurable pipelines with processors and sinks:

```
Webhook → Event Extraction → Processors (Filter, Map) → Sinks (Database)
```

### Processors

#### Filter Processor

Uses CEL (Common Expression Language) to filter events. Only events matching the expression pass through.

**Available variables:**
- `eventType` - The event type (e.g., "jira:issue_created")
- `body` - The entire event body as JSON
- All top-level fields from the body

**Examples:**
```json
{
  "type": "filter",
  "celExpression": "eventType == 'jira:version_updated'"
}
```

```json
{
  "type": "filter",
  "celExpression": "eventType == 'jira:issue_created' && issue.fields.priority == 'High'"
}
```

#### Mapper Processor

Uses Handlebars templates to transform event data into a new structure.

**Example:**
```json
{
  "type": "mapper",
  "outputEvent": {
    "deploymentName": "{{ version.name }}",
    "projectKey": "{{ version.projectId }}",
    "status": "{{ version.released }}",
    "timestamp": "{{ timestamp }}",
    "metadata": {
      "source": "jira",
      "type": "version_update"
    }
  }
}
```

### Sinks

#### Database Sink (MongoDB)

Writes processed events to MongoDB with upsert support.

**Configuration:**
```json
{
  "type": "database",
  "provider": "MONGO"
}
```

**Document structure:**
- `_id` - Event ID (SHA256 hash of primary keys)
- `_eventType` - Original event type
- All fields from the mapped event body

**Operations:**
- `Write` operations use `replace_one` with upsert
- `Delete` operations remove the document by `_id`

## Multiple Pipelines

You can configure multiple pipelines per integration to process events differently:

```json
{
  "pipelines": [
    {
      "processors": [
        {
          "type": "filter",
          "celExpression": "eventType == 'jira:issue_created'"
        }
      ],
      "sinks": [
        {
          "type": "database",
          "provider": "MONGO"
        }
      ]
    },
    {
      "processors": [
        {
          "type": "filter",
          "celExpression": "eventType == 'jira:version_updated'"
        },
        {
          "type": "mapper",
          "outputEvent": {
            "version": "{{ version.name }}",
            "released": "{{ version.released }}"
          }
        }
      ],
      "sinks": [
        {
          "type": "database",
          "provider": "MONGO"
        }
      ]
    }
  ]
}
```

## Development

### Quick Commands (Makefile)

```bash
make help          # Show all available commands
make test          # Run unit tests
make e2e           # Run E2E tests
make build         # Build release binary
make docker        # Build Docker image
make all           # Run formatting, linting, tests, and build
```

### Running Tests

**Unit Tests:**
```bash
cargo test
# or
make test
```

**End-to-End Tests:**

E2E tests run the full stack (ConnectCare + MongoDB) with Docker Compose and test real webhook scenarios.

```bash
# Run E2E tests
./run_e2e_tests.sh
# or
make e2e

# Start E2E environment for manual testing
make e2e-start

# View logs
make e2e-logs

# Stop E2E environment
make e2e-stop
```

**What E2E tests cover:**
- Health check endpoints
- Valid webhook requests with HMAC signatures
- Invalid signatures and missing headers
- Malformed JSON payloads
- Event filtering with CEL expressions
- Event mapping with Handlebars
- MongoDB persistence and data validation
- Delete operations

See [tests/e2e/README.md](tests/e2e/README.md) for detailed documentation.

### Running with Debug Logging

```bash
RUST_LOG=debug cargo run
```

## Security

- HMAC-SHA256 signature validation with constant-time comparison
- Signature format: `sha256=<hex_signature>`
- Configurable signature header name (default: `X-Hub-Signature`)

## Architecture

```
HTTP Request → HMAC Validation → Event Extraction → Pipeline Processing → Sinks
                                                            ↓
                                                    Filter (CEL)
                                                            ↓
                                                    Mapper (Handlebars)
                                                            ↓
                                                    Sink (MongoDB)
```

### Pipeline Event Structure

Each event is normalized to:
```rust
{
  id: String,           // SHA256 hash of primary keys
  body: Value,          // Full JSON payload (or mapped output)
  event_type: String,   // e.g., "jira:issue_updated"
  pk_fields: Vec<...>,  // Primary key fields
  operation: Write|Delete
}
```

### Extensibility

The architecture is designed for extensibility:

- **Processors**: Implement the `Processor` trait to add new processing logic
- **Sinks**: Implement the `Sink` trait to add new destination types
- **Sources**: Add new webhook sources by implementing event extraction

Future sink types could include: HTTP endpoints, Kafka, SQS, etc.

## Docker

### Dockerfile Features

- **Multi-stage build** for minimal image size
- **Distroless base image** (`gcr.io/distroless/cc-debian12`) for security
- **Optimized layer caching** for faster rebuilds
- **No shell or package manager** in final image (security hardening)

### Docker Compose Stack

The `docker-compose.yml` provides a complete stack:
- **connectcare** service on port 8080
- **mongodb** service on port 27017
- Automatic network configuration
- Volume persistence for MongoDB data
- Health checks and restart policies

### Building

```bash
# Build the image
docker build -t connectcare:latest .

# Build with specific tag
docker build -t myregistry/connectcare:v1.0.0 .
```

### Running

```bash
# Run with Docker
docker run -d \
  --name connectcare \
  -p 8080:8080 \
  -e JIRA_WEBHOOK_SECRET="your-secret" \
  -e RUST_LOG=info \
  -v $(pwd)/config/config.json:/app/config/config.json:ro \
  connectcare:latest

# Run with Docker Compose (includes MongoDB)
docker-compose up -d

# View logs
docker-compose logs -f

# Check health
curl http://localhost:8080/-/healthz
```

### Environment Variables

- `RUST_LOG` - Logging level (default: `info`)
- `CONFIGURATION_PATH` - Config file path (default: `/app/config/config.json`)
- `JIRA_WEBHOOK_SECRET` - Jira webhook secret (if using env-based secrets)

### Volumes

Mount your configuration file:
```bash
-v /path/to/your/config.json:/app/config/config.json:ro
```

## License

MIT
