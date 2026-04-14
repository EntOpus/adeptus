FROM docker.io/rust:1.93-alpine3.23 AS builder

WORKDIR /app

# Install musl build dependencies (no openssl — sqlx uses rustls)
RUN apk add --no-cache musl-dev

# Set up fully static musl build
ENV RUSTFLAGS="-C target-feature=+crt-static"

# Copy Cargo files first for dependency caching
COPY Cargo.toml ./

# Create dummy source files to build dependencies
RUN mkdir -p src && echo "pub fn dummy() {}" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release 2>/dev/null || true

# Remove dummy sources
RUN rm -rf src

# Copy real source code, migrations, and config
COPY src/ src/
COPY migrations/ migrations/
COPY config/ config/

# Build the application
RUN cargo build --release --bin adeptus

# --- Runtime stage ---
FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/release/adeptus /adeptus
COPY --from=builder /app/config/ /config/

USER 65534:65534

EXPOSE 3000

ENTRYPOINT ["/adeptus"]
