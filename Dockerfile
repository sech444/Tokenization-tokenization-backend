# ================================
# Builder stage
# ================================
FROM rustlang/rust:nightly-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY . .

# Build for regular Linux target (no musl complications)
RUN cargo build --release && \
    strip target/release/tokenization-backend

# ================================
# Runtime stage
# ================================
FROM debian:bookworm-slim
WORKDIR /app

# Install minimal runtime deps
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -u 1001 -m appuser

# Copy binary and migrations
COPY --from=builder /app/target/release/tokenization-backend /usr/local/bin/
COPY --from=builder /app/migrations ./migrations

# Set ownership
RUN chown -R appuser:appuser ./migrations

USER appuser

# Environment variables
ENV PORT=8080
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT:-8080}/health || exit 1

EXPOSE 8080
CMD ["tokenization-backend"]