FROM rust:1.87-slim as builder

WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    cargo build --release

# ---

FROM debian:bookworm-slim

# Install rtl_433 and dependencies
RUN apt-get update && apt-get install -y rtl-433

WORKDIR /app

# Copy only the compiled binary
COPY --from=builder /app/target/release/meteorolog .

CMD ["./meteorolog"]