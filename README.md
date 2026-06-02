# Flux Cookbook

Executable examples for Flux.

This repository reads a user from MongoDB and stores a transformed version in PostgreSQL.

## Run

```text
docker compose up --build cookbook
```

## Environment

Copy `.env.example` to `.env` and adjust values if needed.

## Layout

```text
flux-cookbook/
  Cargo.toml
  src/
  Dockerfile
  docker-compose.yml
  .env.example
```
