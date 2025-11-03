# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder
WORKDIR /app

# Cache deps first
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release || true

# Copy real sources
COPY src ./src
COPY migrations ./migrations

# Build all binaries
RUN cargo build --release --bins

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy binaries
COPY --from=builder /app/target/release/fast-lottery-engine /usr/local/bin/fast-lottery-engine
COPY --from=builder /app/target/release/migrate /usr/local/bin/migrate
COPY --from=builder /app/target/release/db_prepare /usr/local/bin/db_prepare

# Entrypoint script
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

EXPOSE 8080
ENTRYPOINT ["/docker-entrypoint.sh"]
