# ================================
# Builder stage
# ================================
FROM rustlang/rust:nightly-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl build-essential ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files and build deps
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# ================================
# Runtime stage (production)
# ================================
FROM debian:bullseye-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/tokenization-backend /usr/local/bin/app

CMD ["app"]

# ================================
# Development stage (live reload)
# ================================
FROM rustlang/rust:nightly-slim AS development
WORKDIR /app

# Install dev dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl build-essential ca-certificates \
    && rm -rf /var/lib/apt/lists/*


# Install developer tools
RUN cargo install cargo-watch && \
    cargo install sqlx-cli --no-default-features --features postgres

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY setup-env.sh ./

# Make the setup script executable
RUN chmod +x setup-env.sh

# Run backend with hot reload
CMD ["bash", "-c", "source ./setup-env.sh && cargo watch -x run"]