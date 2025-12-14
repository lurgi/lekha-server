# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/

# Create dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p migration/src && \
    echo "fn main() {}" > migration/src/main.rs && \
    echo "fn main() {}" > migration/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release && \
    rm -rf src migration/src target/release/inklings-server*

# Copy source code
COPY src ./src
COPY migration/src ./migration/src

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/inklings-server .

# Create non-root user
RUN useradd -m -u 1001 appuser && \
    chown -R appuser:appuser /app

USER appuser

EXPOSE 8080

CMD ["./inklings-server"]
