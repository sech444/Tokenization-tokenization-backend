# Multi-stage Dockerfile for Rust Tokenization Backend

# =============================================================================
# Builder Stage - Build the application
# =============================================================================
FROM docker.io/rust:1.75-slim-bullseye AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install sqlx-cli for database migrations
RUN cargo install sqlx-cli --no-default-features --features postgres

# Create app user and group
RUN groupadd -r app && useradd -r -g app app

# Set working directory
WORKDIR /app

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"Dummy main for dependency caching\");}" > src/main.rs && \
    echo "// Dummy lib for dependency caching" > src/lib.rs

# Build dependencies (this layer will be cached if Cargo.toml doesn't change)
RUN cargo build --release && \
    rm -rf src target/release/deps/tokenization_backend*

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# =============================================================================
# Development Stage - For local development with hot reloading
# =============================================================================
FROM docker.io/rust:1.75-slim-bullseye AS development

# Install system dependencies and development tools
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    curl \
    build-essential \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Install development tools
RUN cargo install cargo-watch sqlx-cli --no-default-features --features postgres

# Create app user and group
RUN groupadd -r app && useradd -r -g app app

# Set working directory
WORKDIR /app

# Create necessary directories
RUN mkdir -p /app/storage /app/logs && \
    chown -R app:app /app

# Copy Cargo files
COPY --chown=app:app Cargo.toml Cargo.lock ./

# Create target directory for caching
RUN mkdir -p target && chown -R app:app target

# Switch to app user
USER app

# Expose ports
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command for development (with hot reloading)
CMD ["cargo", "watch", "-x", "run"]

# =============================================================================
# Production Runtime Stage - Minimal runtime image
# =============================================================================
FROM docker.io/debian:bullseye-slim AS production

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    libpq5 \
    curl \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create app user and group with specific UID/GID
RUN groupadd -r -g 1000 app && \
    useradd -r -u 1000 -g app -s /bin/bash -d /app app

# Set working directory
WORKDIR /app

# Create necessary directories
RUN mkdir -p /app/storage /app/logs /app/migrations && \
    chown -R app:app /app

# Copy the binary from builder stage
COPY --from=builder --chown=app:app /app/target/release/tokenization-backend /app/tokenization-backend
COPY --from=builder --chown=app:app /root/.cargo/bin/sqlx /usr/local/bin/sqlx

# Copy migrations
COPY --chown=app:app migrations /app/migrations

# Copy configuration files if they exist
COPY --chown=app:app config* /app/ 2>/dev/null || true

# Switch to app user
USER app

# Expose ports
EXPOSE 8080 9090

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=0

# Run the application
CMD ["/app/tokenization-backend"]

# =============================================================================
# Testing Stage - For running tests
# =============================================================================
FROM builder AS testing

# Install additional testing dependencies
RUN apt-get update && apt-get install -y \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy test configuration
COPY tests ./tests

# Set environment for testing
ENV RUST_ENV=test
ENV RUST_LOG=debug

# Default command for testing
CMD ["cargo", "test", "--release"]

# =============================================================================
# Migration Stage - For running database migrations
# =============================================================================
FROM docker.io/debian:bullseye-slim AS migrate

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    libpq5 \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy sqlx binary from builder
COPY --from=builder /root/.cargo/bin/sqlx /usr/local/bin/sqlx

# Create app user
RUN groupadd -r app && useradd -r -g app app

# Set working directory
WORKDIR /app

# Copy migrations
COPY --chown=app:app migrations ./migrations

# Switch to app user
USER app

# Default command for migrations
CMD ["sqlx", "migrate", "run"]

# =============================================================================
# Build Arguments and Labels
# =============================================================================
ARG BUILD_DATE
ARG VCS_REF
ARG VERSION

LABEL org.label-schema.build-date=$BUILD_DATE \
      org.label-schema.name="tokenization-backend" \
      org.label-schema.description="Rust backend for tokenization platform" \
      org.label-schema.url="https://tokenization.com" \
      org.label-schema.vcs-ref=$VCS_REF \
      org.label-schema.vcs-url="https://github.com/your-org/tokenization-backend" \
      org.label-schema.vendor="Tokenization Platform" \
      org.label-schema.version=$VERSION \
      org.label-schema.schema-version="1.0"
