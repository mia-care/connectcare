# Build stage
FROM rust:1.91.1-slim AS builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy the actual source code
COPY . .

# Build the application (dependencies are already cached)
RUN cargo build --release

# Runtime stage - using distroless
FROM gcr.io/distroless/cc-debian12

# Copy the binary from builder
COPY --from=builder /app/target/release/connectcare /app/connectcare

# Copy default config (can be overridden with volume mount)
COPY config/config.example.json /app/config/config.json

WORKDIR /app

# Expose port
EXPOSE 3000

# Set default environment variables
ENV LOG_LEVEL=info
ENV CONFIGURATION_PATH=/app/config/config.json

# Run the binary
ENTRYPOINT ["/app/connectcare"]
