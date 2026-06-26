use actix_web::web;
use actix_web::web::Data;
use r2d2_postgres::postgres::Transaction;
use crate::dao::r2d2_postgres::connection::DbPool;
use crate::dao::sync::r2d2_pg_backend::PgTran;
use crate::model::result::{DbResult, ErrorCode, WebResult};

pub fn invoke<F, R>(pool: &DbPool, f: F) -> WebResult<R>
    where
        F: FnOnce(&mut Transaction) -> DbResult<R> + Send + 'static,
        R: Send + 'static + Clone,
{
    let mut connection = match pool.get() {
        Ok(c) => c,
        Err(e) => return WebResult::fail(1, &e.to_string()),
    };
    let mut transaction = match connection.transaction() {
        Ok(t) => t,
        Err(e) => return WebResult::fail(1, &e.to_string()),
    };
    let result = f(&mut transaction);
    match result {
        Ok(e) => {
            if let Err(err) = transaction.commit() {
                return WebResult::fail(1, &err.to_string());
            }
            WebResult::success(e)
        }
        Err(e) => {
            let _ = transaction.rollback();
            WebResult::fail(1, format!("{}", e).as_str())
        }
    }
}

pub async fn invoke_block<F, R>(pool: Data<DbPool>, f: F) -> WebResult<R>
    where
        F: FnOnce(&mut Transaction) -> DbResult<R> + Send + 'static,
        R: Send + 'static + Clone {
    match web::block(move || {
        invoke(&pool, f)
    }).await {
        Ok(result) => result,
        Err(e) => WebResult::fail(1, &e.to_string()),
    }
}

/// Invoke a unified DAO via [`PgTran`] with automatic transaction management.
pub async fn invoke_unified<T, F>(pool: Data<DbPool>, f: F) -> WebResult<T>
where
    T: Clone + Send + 'static,
    F: FnOnce(&mut PgTran) -> DbResult<T> + Send + 'static,
{
    let result = match web::block(move || -> Result<T, ErrorCode> {
        let mut connection = pool.get().map_err(|e| ErrorCode::new(1, &e.to_string()))?;
        let mut transaction = connection.transaction().map_err(|e| ErrorCode::new(1, &e.to_string()))?;
        let result = {
            let mut wrapper = PgTran::new(&mut transaction);
            f(&mut wrapper)
        }?;
        transaction.commit().map_err(|e| ErrorCode::new(1, &e.to_string()))?;
        Ok(result)
    })
    .await
    {
        Ok(Ok(r)) => WebResult::success(r),
        Ok(Err(e)) => WebResult::fail(e.code, &e.message),
        Err(e) => WebResult::fail(1, &e.to_string()),
    };
    result
}
