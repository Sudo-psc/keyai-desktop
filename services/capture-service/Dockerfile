# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Create app directory
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies (this is cached as long as Cargo.toml doesn't change)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    libstdc++ \
    openssl

# Create non-root user
RUN addgroup -g 1000 keyai && \
    adduser -D -u 1000 -G keyai keyai

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/capture-service /usr/local/bin/capture-service

# Set ownership
RUN chown keyai:keyai /usr/local/bin/capture-service

# Switch to non-root user
USER keyai

# Expose ports
EXPOSE 8080 9090 50051

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -q --spider http://localhost:8080/health || exit 1

# Environment variables with defaults
ENV CAPTURE_SERVICE_NAME=capture-service \
    CAPTURE_NATS_URL=nats://nats:4222 \
    CAPTURE_NATS_SUBJECT=keyai.events.capture \
    CAPTURE_HEALTH_PORT=8080 \
    CAPTURE_METRICS_PORT=9090 \
    RUST_LOG=capture_service=info

# Run the service
ENTRYPOINT ["/usr/local/bin/capture-service"] 