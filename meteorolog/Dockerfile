# ------------------------
# Builder Stage
# ------------------------
FROM rust:1.87-slim as builder

WORKDIR /app

# 1. Install build dependencies early (caches them)
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# 2. Copy just the manifests to cache dependencies
COPY Cargo.toml Cargo.lock ./

# 3. Dummy build to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -r src

# 4. Copy the full source and do the real build
COPY . .
RUN cargo build --release

# ------------------------
# Runtime Stage
# ------------------------
FROM debian:bookworm-slim

# Install rtl_433 and dependencies
RUN apt-get update && apt-get install -y rtl-433 && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the final binary from builder
COPY --from=builder /app/target/release/meteorolog .

CMD ["./meteorolog"]