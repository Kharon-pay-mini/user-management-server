# === Build stage ===
FROM rust:1.87-slim AS builder

ENV CARGO_BIN_DIR=/cargo-bin

# Install dependencies including CA certificates
RUN apt-get update && apt-get install -y \
    pkg-config libpq-dev build-essential libssl-dev curl \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create a clean cargo bin dir
RUN mkdir -p $CARGO_BIN_DIR
ENV PATH=$CARGO_BIN_DIR:$PATH
ENV CARGO_HOME=/cargo-home

# Install diesel CLI into $CARGO_BIN_DIR
RUN cargo install diesel_cli --no-default-features --features postgres --root $CARGO_BIN_DIR

WORKDIR /app

COPY Cargo.toml ./
COPY Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src


COPY src ./src
COPY migrations ./migrations
COPY diesel.toml diesel.toml

RUN cargo build --release


# === Runtime stage ===
FROM debian:bookworm-slim

# Install runtime dependencies including CA certificates and SSL
RUN apt-get update && apt-get install -y \
    libpq5 \
    openssl \
    ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -m appuser

RUN update-ca-certificates

COPY --from=builder /app/target/release/user-management-server /usr/local/bin/app
COPY --from=builder /cargo-bin/bin/diesel /usr/local/bin/diesel
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# COPY .env /app/.env

# ✅ Sanity check to ensure binary is present
RUN test -x /usr/local/bin/app || (echo "❌ Binary missing!" && exit 1)


USER appuser
WORKDIR /app

ENV RUST_LOG=info

EXPOSE 8001

CMD ["/usr/local/bin/app"]
