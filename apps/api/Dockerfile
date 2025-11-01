# Multi-stage Dockerfile for optimized production build

# Stage 1: Builder
FROM rust:1.82-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml ./
COPY apps/api/Cargo.toml apps/api/

# Create dummy main to cache dependencies
RUN mkdir -p apps/api/src && \
    echo "fn main() {}" > apps/api/src/main.rs && \
    echo "pub fn lib() {}" > apps/api/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release --manifest-path apps/api/Cargo.toml

# Remove dummy files
RUN rm -rf apps/api/src

# Copy actual source code
COPY apps/api/src apps/api/src
COPY apps/api/migrations apps/api/migrations

# Build the actual application
RUN cargo build --release --manifest-path apps/api/Cargo.toml

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/vibe-api /app/vibe-api

# Copy migrations
COPY --from=builder /app/apps/api/migrations /app/migrations

# Create non-root user
RUN useradd -m -u 1001 appuser && \
    chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["/app/vibe-api"]
