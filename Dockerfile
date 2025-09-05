# Multi-stage Dockerfile for Synapse MCP
# Stage 1: Builder - compile the Rust application
FROM rust:1.80 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy the actual source code
COPY src ./src
COPY benches ./benches
COPY tests ./tests

# Build the application
RUN cargo build --release

# Stage 2: Runner - minimal image with the compiled binary
FROM debian:bookworm-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd -r synapse && useradd -r -g synapse synapse

# Create directories for synapse
RUN mkdir -p /app /data && chown -R synapse:synapse /app /data

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/release/synapse /usr/local/bin/synapse

# Make sure the binary is executable
RUN chmod +x /usr/local/bin/synapse

# Switch to non-root user
USER synapse

# Set working directory
WORKDIR /app

# Environment variables
ENV RUST_LOG=info
ENV SYNAPSE_DATA_DIR=/data

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD synapse status || exit 1

# Expose default MCP server port
EXPOSE 8080

# Default command
ENTRYPOINT ["/usr/local/bin/synapse"]
CMD ["serve", "--host", "0.0.0.0", "--port", "8080"]