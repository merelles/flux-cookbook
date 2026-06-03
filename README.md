# Flux MongoDB To PostgreSQL

Executable Flux example that copies a user from MongoDB into PostgreSQL.

This example consumes Flux crates from the Git repository instead of local path dependencies.

## Run

```text
docker compose up --build mongodb-to-postgres
```

## Environment

Copy `.env.example` to `.env` and adjust values if needed.

## Layout

```text
flux-mongodb-to-postgres/
  Cargo.toml
  src/
  Dockerfile
  docker-compose.yml
  .env.example
```
