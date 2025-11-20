# Integration Connector Agent - Rust

A high-performance webhook receiver service for Jira integrations, written in Rust.

## Features

- ✅ Jira webhook integration with HMAC-SHA256 signature validation
- ✅ Support for 15+ Jira event types (issues, projects, versions, issue links)
- ✅ Secure secret management (environment variables, files, or plain text)
- ✅ Event normalization and pipeline processing
- ✅ Health check endpoints
- ✅ Structured logging with tracing

## Quick Start

### Prerequisites

- Rust 1.70+ 
- Cargo

### Installation

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

3. Update `config/config.json` as needed.

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
      }
    }
  ]
}
```

## Development

### Running Tests

```bash
cargo test
```

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
HTTP Request → HMAC Validation → Event Extraction → Pipeline → Processing
```

### Pipeline Event Structure

Each event is normalized to:
```rust
{
  id: String,           // SHA256 hash of primary keys
  body: Value,          // Full JSON payload
  event_type: String,   // e.g., "jira:issue_updated"
  pk_fields: Vec<...>,  // Primary key fields
  operation: Write|Delete
}
```

## License

MIT
