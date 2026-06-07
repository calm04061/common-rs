# AGENTS.md

## Build & verify

```bash
# build (default features: r2d2_pg)
cargo build

# check without building
cargo check

# test default features
cargo test

# run with specific database backend
cargo build --features sqlx_pg         # sqlx + async pg
cargo build --features sqlite          # r2d2 + sqlite
cargo build --features sqlx_oracle     # sqlx + oracle
cargo build --features quartz          # quartz scheduler

# combine features
cargo build --features "sqlx_pg quartz"
```

## Architecture

Single-crate library (`edition 2024`, no workspace). Feature-gated modules:

- **`r2d2_pg`** (default) — sync r2d2 + tokio-postgres connection pool + blocking DAO. Use `web::r2d2_postgres::invoke_block` for actix-web endpoints.
- **`sqlx_pg`** — async sqlx postgres pool. Use `web::sqlx_postgres::invoke` for actix-web.
- **`sqlite`** — r2d2 + sqlite, `DATABASE_URL` env or fallback `/var/lib/cloud/config/file.db`.
- **`sqlx_oracle`** — sqlx + Oracle via custom `sqlx-oracle` fork (git dep, branch `feature/sqlx9`).
- **`quartz`** — `quartz_sched` scheduler, gated behind `lazy_static`. Access via `get_scheduler()`.

## Library, not binary

`common-rs` is a **library crate** (no `main.rs`). Consumed as a dependency by other projects for shared DB/web/scheduler infrastructure.

## Key conventions

- **Rust edition 2024** (uncommon — check edition-specific syntax before adding code).
- **No nightly features assumed** despite edition 2024.
- **`dotenv` used at runtime** for `DATABASE_URL` — always check `.env` or env vars for DB config.
- **`lazy_static`** used for singletons (scheduler, sqlite connection) instead of `once_cell` or `std::sync::OnceLock`.
- **gitflow**: `production_branch = "main"`, `development_branch = "develop"`, prefix `feature/`, `hotfix/`, tag `v`.
- **DAO traits are sync** (r2d2_pg, sqlite) or **async** (sqlx_pg, sqlx_oracle). The sync `SimpleDao` lives at `dao::r2d2_postgres::dao::SimpleDao` and `dao::sqlite::dao::SimpleDao`. The async version lives at `dao::sqlx_postgres::SimpleDao`.
- **`WebResult` / `DbResult`** custom types in `model::result`. Conversions via `From` impls — prefer `.into()`.

## What not to do

- Do not add a `main.rs` or a binary target without confirmation.
- Do not add `once_cell`, `std::sync::OnceLock`, or replace `lazy_static` — the crate uses `lazy_static` consistently.
- Do not add `tokio-postgres` or `r2d2_postgres` for non-default features — they are `r2d2_pg`-only.
- Do not commit `Cargo.lock` (it's in `.gitignore`).