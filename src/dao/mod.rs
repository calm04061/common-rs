// ---------------------------------------------------------------------------
// Old per-backend modules — kept for backward compatibility.
// ---------------------------------------------------------------------------
#[cfg(all(feature = "sqlx_oracle", feature = "sqlx_pg"))]
mod sqlx_dao;
#[cfg(feature = "r2d2_pg")]
pub mod r2d2_postgres;
#[cfg(feature = "sqlx_pg")]
pub mod sqlx_postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;
#[cfg(feature = "sqlx_oracle")]
pub mod sqlx_oracle;

// ---------------------------------------------------------------------------
// New unified modules (方案三).
//
// `sync`  — shared by r2d2_pg + sqlite  (sync/blocking backends)
// `async` — shared by sqlx_pg + sqlx_oracle  (async backends)
// ---------------------------------------------------------------------------

/// Unified sync DAO — available when at least one sync backend is enabled.
#[cfg(any(feature = "r2d2_pg", feature = "sqlite"))]
pub mod sync;

/// Unified async DAO — available when at least one sqlx backend is enabled.
#[cfg(any(feature = "sqlx_pg", feature = "sqlx_oracle"))]
pub mod r#async;

// Re-exports for ergonomic usage
#[cfg(any(feature = "r2d2_pg", feature = "sqlite"))]
pub use sync::SimpleDao;
#[cfg(any(feature = "sqlx_pg", feature = "sqlx_oracle"))]
pub use r#async::AsyncSimpleDao;