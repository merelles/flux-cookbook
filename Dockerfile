FROM rust:1-bookworm AS builder

WORKDIR /app

COPY . ./flux-cookbook

WORKDIR /app/flux-cookbook

ARG EXAMPLE_PACKAGE=flux-mongodb-to-postgres
RUN cargo build --release -p ${EXAMPLE_PACKAGE}

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/flux-cookbook/target/release/flux-mongodb-to-postgres /usr/local/bin/flux-mongodb-to-postgres

CMD ["flux-mongodb-to-postgres"]
