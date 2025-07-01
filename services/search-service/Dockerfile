# Build stage
FROM rust:1.78 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Copy source
COPY src ./src

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/search-service /app/search-service

# Expose port
EXPOSE 8004

# Run the binary
CMD ["./search-service"] 