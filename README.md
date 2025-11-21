# Integration Connector Agent - Rust

A high-performance webhook receiver service for Jira integrations with configurable event processing pipelines and MongoDB sink support, written in Rust.

## Features

- ✅ Jira webhook integration with HMAC-SHA256 signature validation
- ✅ Support for 15+ Jira event types (issues, projects, versions, issue links)
- ✅ Secure secret management (environment variables, files, or plain text)
- ✅ Event normalization and pipeline processing
- ✅ **CEL (Common Expression Language) filters** for event filtering
- ✅ **Handlebars template-based event mapping**
- ✅ **MongoDB sink** for persisted event storage
- ✅ Extensible processor and sink architecture
- ✅ Health check endpoints
- ✅ Structured logging with tracing

## Quick Start

### Prerequisites

- Rust 1.70+ 
- Cargo
- MongoDB (if using database sink)

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

3. Configure MongoDB connection (optional):
```bash
# Update config.json with your MongoDB connection details
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
  "mongodb": {
    "connection_string": "mongodb://localhost:27017",
    "database": "connectcare",
    "collection": "events"
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

## License

MIT
