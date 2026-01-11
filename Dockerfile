# ---------- Build Stage ----------
FROM rust:1.88-slim AS builder

WORKDIR /app

# Install musl target
RUN rustup target add x86_64-unknown-linux-musl

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Build actual application
COPY ./src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl --bin pong-server

# ---------- Runtime Stage ----------
FROM gcr.io/distroless/static

WORKDIR /app

# Copy statically linked binary
COPY --from=builder \
  /app/target/x86_64-unknown-linux-musl/release/pong-server \
  /pong-server

EXPOSE 7777

ENTRYPOINT ["/pong-server"]
