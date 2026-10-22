# Build context is the kit root: compose uses `context: .` and `dockerfile: backend/Dockerfile`.
FROM rust:1 AS builder
WORKDIR /build/backend

# Cargo.toml lists [[test]] targets under ../tests/backend, so those files must exist.
COPY backend/ /build/backend/
# The shared project scaffold (path dependency ../engine/build) and the starter project it embeds.
COPY engine/build/ /build/engine/build/
COPY engine/starter/ /build/engine/starter/
# The add-ons, compiled into xerxes_build (its build.rs reads ../addons).
COPY engine/addons/ /build/engine/addons/
COPY tests/backend/ /build/tests/backend/

# Cargo.lock is copied with backend/ (--locked is not used, so a stale lock is refreshed).
RUN cargo build --release --bins

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home app

WORKDIR /app
COPY --from=builder /build/backend/target/release/api /app/api
COPY --from=builder /build/backend/target/release/worker /app/worker
COPY backend/migrations /app/migrations

ENV MIGRATIONS_DIR=/app/migrations
USER app
EXPOSE 8000
CMD ["/app/api"]
