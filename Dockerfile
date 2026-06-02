FROM rust:1-bookworm AS builder

WORKDIR /app

COPY flux/Cargo.toml flux/Cargo.lock ./flux/
COPY flux/flux ./flux/flux
COPY flux/flux-derive ./flux/flux-derive
COPY flux/flux-postgres ./flux/flux-postgres
COPY flux/flux-mongodb ./flux/flux-mongodb
COPY flux-cookbook ./flux-cookbook

WORKDIR /app/flux-cookbook

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/flux-cookbook /usr/local/bin/flux-cookbook

CMD ["flux-cookbook"]
