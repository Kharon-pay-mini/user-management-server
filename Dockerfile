# ========================
# 🛠 Build Stage
# ========================
FROM rustlang/rust:nightly-slim AS build

# Install dependencies for PostgreSQL and OpenSSL
RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    pkg-config \
    build-essential \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build the app using cargo (release mode)
RUN cargo build --release

# ========================
# 🚀 Runtime Stage
# ========================
FROM debian:bookworm-slim AS runtime

# Install only runtime dependencies and CA certificates
RUN apt-get update && apt-get install -y \
    libpq-dev \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from build stage
COPY --from=build /app/target/release/user-management-server /usr/local/bin/app

# Set runtime environment variables
ENV RUST_LOG=info

# Run the binary
ENTRYPOINT ["app"]
