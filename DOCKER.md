# Docker Quick Start Guide

This guide helps you quickly get ConnectCare running using Docker.

## Prerequisites

- Docker 20.10+
- Docker Compose 2.0+ (for full stack)

## Quick Start with Docker Compose

1. **Clone the repository** (or ensure you have the project files)

2. **Set your Jira webhook secret:**
   ```bash
   export JIRA_WEBHOOK_SECRET="your-secret-here"
   ```

3. **Configure the application:**
   ```bash
   cp config/config.example.json config/config.json
   # Edit config.json as needed
   ```
   
   **Important:** When using docker-compose, make sure the MongoDB connection string uses `mongodb://mongodb:27017` (the service name, not localhost).

4. **Start all services:**
   ```bash
   docker-compose up -d
   ```

5. **Check the logs:**
   ```bash
   docker-compose logs -f connectcare
   ```

6. **Verify it's running:**
   ```bash
   curl http://localhost:8080/-/healthz
   ```

## Docker Only (Without MongoDB)

If you have MongoDB running elsewhere or don't need the database sink:

```bash
# Build the image
docker build -t connectcare:latest .

# Run the container
docker run -d \
  --name connectcare \
  -p 8080:8080 \
  -e JIRA_WEBHOOK_SECRET="your-secret-here" \
  -e LOG_LEVEL=info \
  -v $(pwd)/config/config.json:/app/config/config.json:ro \
  connectcare:latest

# View logs
docker logs -f connectcare
```

## Configuration Notes

### MongoDB Connection String

- **With docker-compose:** Use `mongodb://mongodb:27017` (service name)
- **With external MongoDB:** Use `mongodb://host.docker.internal:27017` (from container to host)
- **With remote MongoDB:** Use the full connection string

### Environment Variables

- `JIRA_WEBHOOK_SECRET` - Your Jira webhook secret
- `LOG_LEVEL` - Logging level (debug, info, warn, error)
- `CONFIGURATION_PATH` - Path to config file (default: `/app/config/config.json`)
- `MONGO_URL` - MongoDB connection string (optional, can also be configured per-sink in config file)

### Volume Mounts

The default docker-compose setup mounts:
- `./config/config.json` → `/app/config/config.json` (read-only)

You can add additional mounts as needed (e.g., for file-based secrets).

## Useful Commands

```bash
# Start services
docker-compose up -d

# Stop services
docker-compose down

# View logs (all services)
docker-compose logs -f

# View logs (specific service)
docker-compose logs -f connectcare

# Restart a service
docker-compose restart connectcare

# Check service status
docker-compose ps

# Rebuild after code changes
docker-compose up -d --build

# Clean up everything (including volumes)
docker-compose down -v
```

## Testing

Test the webhook endpoint:

```bash
# Generate a test signature
echo -n '{"test":"data"}' | openssl dgst -sha256 -hmac "your-secret-here" | cut -d' ' -f2

# Send a test request
curl -X POST http://localhost:8080/jira/webhook \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature: sha256=<signature-from-above>" \
  -d '{"webhookEvent":"jira:issue_created","issue":{"id":"12345","key":"TEST-1"}}'
```

## Troubleshooting

### Container won't start

Check logs:
```bash
docker-compose logs connectcare
```

### Can't connect to MongoDB

1. Ensure the MongoDB service is running:
   ```bash
   docker-compose ps mongodb
   ```

2. Check MongoDB logs:
   ```bash
   docker-compose logs mongodb
   ```

3. Verify the connection string in `config.json` uses `mongodb://mongodb:27017`

### Configuration not loading

1. Verify the volume mount:
   ```bash
   docker-compose exec connectcare ls -la /app/config/
   ```

2. Check file permissions (should be readable)

3. Verify JSON syntax:
   ```bash
   cat config/config.json | jq .
   ```

## Security Considerations

- **Distroless base image** - No shell, minimal attack surface
- **Non-root user** - Container runs as non-root
- **Read-only config** - Configuration mounted as read-only
- **Secret management** - Use environment variables or mounted secrets, never hardcode

## Production Deployment

For production, consider:

1. **Use a proper secret management system** (AWS Secrets Manager, HashiCorp Vault, etc.)
2. **Set resource limits** in docker-compose.yml
3. **Enable health checks** and monitoring
4. **Use a reverse proxy** (nginx, traefik) with TLS
5. **Configure proper logging** and log aggregation
6. **Use persistent volumes** for MongoDB with backups
7. **Use specific image tags** instead of `latest`

Example with resource limits:

```yaml
services:
  connectcare:
    # ... other config ...
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 256M
```
