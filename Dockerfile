# Build stage
FROM rust:1.89-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (for caching)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src/ ./src/

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Create app directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/Reframe /app/reframe

# Copy configuration file from build context
COPY ./reframe.config.json /app/reframe.config.json

# Create directories for packages and config
RUN mkdir -p /packages /app/config /var/log/reframe

# Download and extract the SWIFT CBPR package
# PACKAGE_URL should be passed as a build argument
ARG PACKAGE_URL
RUN if [ -n "${PACKAGE_URL}" ]; then \
        curl -L -o /tmp/package.zip "${PACKAGE_URL}" && \
        unzip /tmp/package.zip -d /packages && \
        rm /tmp/package.zip && \
        # Rename the extracted directory to match expected name
        (mv /packages/reframe-package-swift-cbpr /packages/swift-cbpr || \
         mv /packages/reframe-swift-cbpr-* /packages/swift-cbpr || \
         true); \
    else \
        echo "Warning: PACKAGE_URL not provided, skipping package download"; \
    fi

# Change ownership to app user
RUN chown -R appuser:appuser /app /packages /var/log/reframe

# Switch to app user
USER appuser

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV TOKIO_WORKER_THREADS=auto
ENV API_SERVER_URL=http://localhost:3000
# Default package path (can be overridden)
ENV REFRAME_PACKAGE_PATH=/packages/swift-cbpr

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["./reframe"]
