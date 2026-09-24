# ========================================================
# Stage 1: Build dependencies and binary with caching
# ========================================================
FROM rust:slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests for layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source tree to pre-build and cache external crates
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/bdai_backend* target/release/bdai_backend*

# Copy real source code and embedded SQL migrations/seeds
COPY src ./src
COPY sql ./sql

# Build optimized production binary
RUN touch src/main.rs && cargo build --release

# Strip binary to reduce final image size
RUN strip target/release/bdai_backend

# ========================================================
# Stage 2: Ultra-lean minimal runtime container
# ========================================================
FROM debian:bookworm-slim AS runner

# Install ca-certificates (required for Supabase SSL & Cloudinary HTTPS) and curl
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Run as non-root user for security best practices
RUN groupadd -g 1001 bdai && \
    useradd -u 1001 -g bdai -m -s /bin/bash bdai

WORKDIR /app

# Copy binary from builder
COPY --from=builder --chown=bdai:bdai /app/target/release/bdai_backend /app/bdai_backend

USER bdai

# Render assigns dynamic PORT environment variable (default 10000 on Render)
ENV PORT=10000
EXPOSE 10000

# Container healthcheck using public health route
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT}/api/health || exit 1

ENTRYPOINT ["/app/bdai_backend"]
