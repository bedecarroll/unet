# Multi-stage Dockerfile for μNet
# Builds both unet-server and unet-cli binaries with optimization

#############################################
# Stage 1: Build Environment
#############################################
FROM rust:1.85-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libgit2-dev \
    zlib1g-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/unet

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY clippy.toml rustfmt.toml ./

# Copy source code structure
COPY crates/ crates/
COPY migrations/ migrations/

# Build dependencies first (better caching)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/unet/target \
    cargo build --release --bin unet-server --bin unet

# Final build with optimizations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/unet/target \
    cargo build --release --bin unet-server --bin unet && \
    cp target/release/unet-server /tmp/unet-server && \
    cp target/release/unet /tmp/unet

#############################################
# Stage 2: Runtime Environment - Server
#############################################
FROM debian:bookworm-slim AS server

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libgit2-1.5 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN useradd --create-home --shell /bin/bash --user-group --uid 1000 unet

# Create application directories
RUN mkdir -p /app/config /app/data /app/logs && \
    chown -R unet:unet /app

# Copy server binary
COPY --from=builder /tmp/unet-server /usr/local/bin/unet-server
RUN chmod +x /usr/local/bin/unet-server

# Copy sample configuration files
COPY config*.toml /app/config/

# Switch to non-root user
USER unet
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose ports
EXPOSE 8080 8443

# Environment variables
ENV RUST_LOG=info
ENV UNET_CONFIG_PATH=/app/config
ENV UNET_DATA_PATH=/app/data
ENV UNET_LOG_PATH=/app/logs

# Default command
CMD ["unet-server"]

#############################################
# Stage 3: Runtime Environment - CLI Tools
#############################################
FROM debian:bookworm-slim AS cli

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libgit2-1.5 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN useradd --create-home --shell /bin/bash --user-group --uid 1000 unet

# Create application directories
RUN mkdir -p /app/config /app/data && \
    chown -R unet:unet /app

# Copy CLI binary
COPY --from=builder /tmp/unet /usr/local/bin/unet
RUN chmod +x /usr/local/bin/unet

# Copy sample configuration files
COPY config*.toml /app/config/

# Switch to non-root user
USER unet
WORKDIR /app

# Environment variables
ENV RUST_LOG=info
ENV UNET_CONFIG_PATH=/app/config
ENV UNET_DATA_PATH=/app/data

# Default command
ENTRYPOINT ["unet"]
CMD ["--help"]

#############################################
# Stage 4: All-in-One Image (Default)
#############################################
FROM debian:bookworm-slim AS all-in-one

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libgit2-1.5 \
    curl \
    supervisor \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN useradd --create-home --shell /bin/bash --user-group --uid 1000 unet

# Create application directories
RUN mkdir -p /app/config /app/data /app/logs /etc/supervisor/conf.d && \
    chown -R unet:unet /app

# Copy both binaries
COPY --from=builder /tmp/unet-server /usr/local/bin/unet-server
COPY --from=builder /tmp/unet /usr/local/bin/unet
RUN chmod +x /usr/local/bin/unet-server /usr/local/bin/unet

# Copy sample configuration files
COPY config*.toml /app/config/

# Create supervisor configuration
RUN cat > /etc/supervisor/conf.d/unet.conf << 'EOF'
[program:unet-server]
command=/usr/local/bin/unet-server
directory=/app
user=unet
autostart=true
autorestart=true
stderr_logfile=/app/logs/unet-server.err.log
stdout_logfile=/app/logs/unet-server.out.log
environment=RUST_LOG=info,UNET_CONFIG_PATH=/app/config,UNET_DATA_PATH=/app/data,UNET_LOG_PATH=/app/logs
EOF

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Expose ports
EXPOSE 8080 8443

# Environment variables
ENV RUST_LOG=info
ENV UNET_CONFIG_PATH=/app/config
ENV UNET_DATA_PATH=/app/data
ENV UNET_LOG_PATH=/app/logs

# Default target is all-in-one
FROM all-in-one

# Switch to non-root user for final image
USER unet
WORKDIR /app

# Default command starts supervisor which manages the server
CMD ["/usr/bin/supervisord", "-n", "-c", "/etc/supervisor/supervisord.conf"]