FROM rust:1-bookworm AS builder

WORKDIR /app

COPY . ./flux-mongodb-to-postgres

WORKDIR /app/flux-mongodb-to-postgres

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/flux-mongodb-to-postgres/target/release/flux-mongodb-to-postgres /usr/local/bin/flux-mongodb-to-postgres

CMD ["flux-mongodb-to-postgres"]
