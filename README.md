# Flux Cookbook

Executable examples for learning Flux in real scenarios.

Each example is its own crate under `examples/`. This keeps scenarios isolated and makes room for CRUD, aggregate, and relationship examples without mixing concerns.

## Examples

- `examples/mongodb-crud-filter`: MongoDB repository basics with filters and pagination.
- `examples/mongodb-to-postgres`: copies a user from MongoDB into PostgreSQL.
- `examples/postgres-bulk`: PostgreSQL bulk write operations.
- `examples/postgres-crud`: basic PostgreSQL repository operations.
- `examples/postgres-generated-id`: PostgreSQL database-generated IDs.
- `examples/postgres-has-many`: one-to-many aggregate persistence.
- `examples/postgres-has-one`: one-to-one aggregate persistence.
- `examples/postgres-many-to-many`: many-to-many aggregate persistence.
- `examples/postgres-pagination-filter`: PostgreSQL pagination and filter AST usage.
- `examples/postgres-transaction-rollback`: aggregate rollback behavior.

## Delivery Map

These examples define the practical surface Flux needs to prove. Each project should be small, executable, and focused on one contract.

1. `examples/postgres-crud`
   Basic `insert`, `find_by_id`, `update`, `save`, `delete`, `exists`, and `count` with PostgreSQL.

2. `examples/postgres-pagination-filter`
   Cursor and offset pagination with `GenericFilter`, ordering, grouped `AND`/`OR`, null checks, dates, and numeric filters.

3. `examples/postgres-bulk`
   `insert_many`, `update_many`, `save_many`, and `delete_many` with realistic batch sizes and chunking.

4. `examples/postgres-generated-id`
   `#[generated_id]` with database-generated primary keys and returned IDs.

5. `examples/postgres-has-one`
   One-to-one aggregate persistence using `#[has_one]`, `find_graph_by_id`, and `save_graph`.

6. `examples/postgres-has-many`
   One-to-many aggregate persistence using `#[has_many]`, `GraphSaveMode::AppendChildren`, `UpsertChildren`, and `ReplaceChildren`.

7. `examples/postgres-many-to-many`
   Many-to-many aggregate persistence using `#[many_to_many]` and join-table replacement.

8. `examples/postgres-transaction-rollback`
   Failing aggregate save that proves rollback behavior and prevents partial graph writes.

9. `examples/mongodb-crud-filter`
   MongoDB repository basics with `MongoEntity`, `MongoObjectId`, BSON filter rendering, pagination, and bulk writes.

10. `examples/mongodb-to-postgres`
    Cross-adapter workflow that reads from MongoDB and writes a transformed row into PostgreSQL.

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
