# Multi-stage Dockerfile for pm4py-rust with security hardening
# Stage 1: Builder (Python dev headers required for PyO3 / pm4py-bridge default feature)
FROM rust:bookworm AS builder

LABEL maintainer="Sean Chatman <info@chatmangpt.com>"
LABEL description="pm4py-rust process mining HTTP server + library - builder stage"

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    python3 \
    python3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Stub lib + bin so `cargo build` can resolve deps before full COPY src (crate has [lib] + main.rs)
RUN mkdir -p src && \
    echo "pub fn _docker_dep_cache() {}" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release 2>&1 | grep -v "warning" || true

# Copy actual source
COPY src ./src
COPY benches ./benches
COPY tests ./tests

# Build the library AND the HTTP server binary (src/main.rs → pm4py)
RUN cargo build --release \
    --features "std,a2a" \
    && cargo test --release --no-run 2>&1 | tail -20

# Verify binary exists
RUN ls -la /build/target/release/

# Stage 2: Runtime
FROM debian:bookworm-slim

LABEL maintainer="Sean Chatman <info@chatmangpt.com>"
LABEL description="pm4py-rust process mining HTTP server + library - runtime"

# PyO3-linked binary needs Python shared library at runtime; curl for healthcheck.
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    python3-minimal \
    libpython3.11 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 -s /sbin/nologin -c "pm4py service user" pm4py && \
    mkdir -p /var/pm4py/logs /var/pm4py/data && \
    chown -R pm4py:pm4py /var/pm4py

# Security: Set restrictive file permissions
RUN chmod -R 750 /var/pm4py && \
    chmod +r /etc/ssl/certs/ca-certificates.crt

WORKDIR /app

# Copy built artifacts from builder (binary + library)
COPY --from=builder --chown=pm4py:pm4py /build/target/release/pm4py /app/pm4py
COPY --from=builder --chown=pm4py:pm4py /build/target/release/libpm4py* /app/
COPY --from=builder --chown=pm4py:pm4py /build/Cargo.toml /build/Cargo.lock /app/

# Copy source (read-only reference)
COPY --from=builder --chown=pm4py:pm4py /build/src /app/src/

# Security hardening: remove SUID/SGID binaries
RUN find / -perm /6000 -type f 2>/dev/null | xargs chmod a-s 2>/dev/null || true

# Switch to non-root user
USER pm4py:pm4py

# Environment configuration
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV PM4PY_DATA_DIR=/var/pm4py/data
ENV PM4PY_LOG_DIR=/var/pm4py/logs
ENV PM4PY_RUST_PORT=8090

# Expose non-privileged port (axum server listens on 8090)
EXPOSE 8090

# Health check — verify the HTTP server is responding
HEALTHCHECK --interval=30s --timeout=3s --start-period=15s --retries=3 \
    CMD curl -sf http://localhost:8090/api/health || exit 1

# Start the pm4py-rust HTTP server
CMD ["/app/pm4py"]
