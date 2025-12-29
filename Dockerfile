# Multi-stage build for Call for Papers application
# This Dockerfile builds both the frontend (Yew.rs) and backend (Rust/Axum)

# Stage 1: Build Frontend
FROM docker.io/library/rust:1.75-slim as frontend-builder

# Install dependencies for WASM compilation
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install WASM target and Trunk
RUN rustup target add wasm32-unknown-unknown && \
    cargo install trunk --version 0.18.8

WORKDIR /app/frontend

# Copy frontend files
COPY frontend/Cargo.toml frontend/Cargo.lock ./
COPY frontend/src ./src
COPY frontend/index.html ./
COPY frontend/Trunk.toml ./

# Build frontend
RUN trunk build --release

# Stage 2: Build Backend
FROM docker.io/library/rust:1.75-slim as backend-builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build backend (release mode)
RUN cargo build --release

# Stage 3: Runtime
FROM docker.io/library/debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled backend binary
COPY --from=backend-builder /app/target/release/call-for-papers ./call-for-papers

# Copy frontend dist files
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist

# Copy migrations
COPY --from=backend-builder /app/migrations ./migrations

# Create non-root user
RUN useradd -m -u 1000 cfp && \
    chown -R cfp:cfp /app

USER cfp

# Expose port
EXPOSE 8080

# Set default environment variables
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV RUST_LOG=call_for_papers=info,tower_http=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# Run the application
CMD ["/app/call-for-papers"]
