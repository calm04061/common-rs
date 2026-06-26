# common-rs

Shared database, web, and scheduler infrastructure for Rust services.

## Features

| Feature | Backend | Sync/Async | Description |
|---------|---------|-----------|-------------|
| `r2d2_pg` (default) | `r2d2_postgres` + `tokio-postgres` | Sync | PostgreSQL connection pool via r2d2 |
| `sqlx_pg` | `sqlx` 0.9 + `postgres` | Async | PostgreSQL via sqlx |
| `sqlite` | `r2d2_sqlite` | Sync | SQLite via r2d2 |
| `sqlx_oracle` | `sqlx` 0.9 + `sqlx-oracle` | Async | Oracle via sqlx fork |
| `quartz` | `quartz_sched` | — | Quartz scheduler (lazy_static) |
| `web` (default) | `actix-web` 4 | — | Actix-web helper functions |

Example:
```bash
cargo build --features "sqlx_pg quartz"     # sqlx pg + quartz
cargo build --features sqlite                # r2d2 sqlite
cargo build --features sqlx_oracle           # sqlx oracle
cargo test                                   # default features
```

## DAO traits

- **`dao::sync::SimpleDao`** — unified sync DAO, works with `r2d2_pg` and `sqlite`
- **`dao::r#async::AsyncSimpleDao`** — unified async DAO, works with `sqlx_pg` and `sqlx_oracle`

Old per-backend `SimpleDao` traits (`dao::r2d2_postgres::dao`, `dao::sqlite::dao`, `dao::sqlx_postgres`, `dao::sqlx_oracle`) are deprecated — use the unified traits instead.

## Web helpers

- `web::r2d2_postgres::invoke_block` — actix-web `web::block` wrapper for sync DAO
- `web::sqlx_postgres::invoke` — async web handler helper

## License

Internal use.
