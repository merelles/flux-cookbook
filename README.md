# Flux Cookbook

Executable examples for learning Flux in real scenarios.

Each example is its own crate under `examples/`. This keeps scenarios isolated and makes room for CRUD, aggregate, and relationship examples without mixing concerns.

## Examples

- `examples/mongodb-to-postgres`: copies a user from MongoDB into PostgreSQL.

Planned examples:

- `examples/postgres-crud`: basic CRUD with Postgres.
- `examples/postgres-aggregate`: aggregate persistence with Postgres.
- `examples/postgres-relations`: one-to-one, one-to-many, and many-to-many relationships.

## Run

Run the MongoDB to PostgreSQL example:

```text
docker compose up --build mongodb-to-postgres
```

Run a crate directly:

```text
cargo run -p flux-mongodb-to-postgres
```

## Environment

Copy `.env.example` to `.env` and adjust values if needed.

## Layout

```text
flux-cookbook/
  Cargo.toml
  examples/
    mongodb-to-postgres/
      Cargo.toml
      src/
  Dockerfile
  docker-compose.yml
  .env.example
```
