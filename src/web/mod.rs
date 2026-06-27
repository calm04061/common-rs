// ---------------------------------------------------------------------------
// Generic helpers available when any sqlx backend is enabled.
// ---------------------------------------------------------------------------
#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
use crate::dao::async_dao::{AsyncTran, Db, DbPool};
#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
use crate::model::result::{DbResult, WebResult};
#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
use actix_web::web::Data;
#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
use sqlx::Transaction;

/// Shorthand: begin a transaction from `DbPool`, run `f`, commit or rollback.
///
/// Works with Postgres (`Transaction<'_, Postgres>`) since `Db = Postgres`
/// for the `sqlx_pg` backend. For MySQL or Oracle use the per-backend
/// modules (`web::sqlx_mysql::invoke`, `web::sqlx_oracle::invoke`) which
/// accept the wrapped transaction types.
#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
pub async fn invoke<F, R>(pool: Data<DbPool>, f: F) -> WebResult<R>
where
    F: FnOnce(&mut Transaction<'_, Db>) -> DbResult<R> + Send + 'static,
    R: Send + 'static + Clone,
{
    let mut transaction = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return WebResult::fail(1, &e.to_string()),
    };
    let result = f(&mut transaction);
    match result {
        Ok(e) => {
            if let Err(err) = transaction.commit().await {
                return WebResult::fail(1, &err.to_string());
            }
            WebResult::success(e)
        }
        Err(e) => {
            let _ = transaction.rollback().await;
            WebResult::fail(1, format!("{}", e).as_str())
        }
    }
}

/// Commit or roll back a transaction based on a `DbResult`.
#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
pub async fn after_transaction<T>(result: DbResult<T>, tran: AsyncTran<'_>) -> DbResult<T> {
    match result {
        Ok(t) => {
            tran.commit().await?;
            Ok(t)
        }
        Err(e) => {
            let _ = tran.rollback().await;
            Err(e)
        }
    }
}
