# Build stage
FROM rust:1.91.1-slim AS builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy all source files
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage - using distroless
FROM gcr.io/distroless/cc-debian12

# Copy the binary from builder
COPY --from=builder /app/target/release/connectcare /app/connectcare

# Copy default config (can be overridden with volume mount)
COPY config/config.example.json /app/config/config.json

WORKDIR /app

# Expose port
EXPOSE 8080

# Set default environment variables
ENV RUST_LOG=info
ENV CONFIGURATION_PATH=/app/config/config.json

# Run the binary
ENTRYPOINT ["/app/connectcare"]
