use actix_web::web;
use actix_web::web::Data;
use r2d2_postgres::postgres::Transaction;
use crate::dao::r2d2_postgres::connection::DbPool;
use crate::dao::sync::r2d2_pg_backend::PgTran;
use crate::model::result::{DbResult, WebResult};

pub fn invoke<F, R>(pool: &DbPool, f: F) -> WebResult<R>
    where
        F: FnOnce(&mut Transaction) -> DbResult<R> + Send + 'static,
        R: Send + 'static + Clone,
{
    let mut connection = pool.get().unwrap();
    let mut transaction = connection.transaction().unwrap();
    let result = f(&mut transaction);
    match result {
        Ok(e) => {
            transaction.commit().unwrap();
            WebResult::success(e)
        }
        Err(e) => {
            transaction.rollback().unwrap();
            WebResult::fail(1, format!("{}", e).as_str())
        }
    }
}

pub async fn invoke_block<F, R>(pool: Data<DbPool>, f: F) -> WebResult<R>
    where
        F: FnOnce(&mut Transaction) -> DbResult<R> + Send + 'static,
        R: Send + 'static + Clone {
    web::block(move || {
        invoke(&pool, f)
    }).await.unwrap()
}

/// Invoke a unified DAO via [`PgTran`] with automatic transaction management.
pub async fn invoke_unified<T, F>(pool: Data<DbPool>, f: F) -> WebResult<T>
where
    T: Clone + Send + 'static,
    F: FnOnce(&mut PgTran) -> DbResult<T> + Send + 'static,
{
    let result = web::block(move || -> DbResult<T> {
        let mut connection = pool.get().unwrap();
        let mut transaction = connection.transaction().unwrap();
        let result = {
            let mut wrapper = PgTran::new(&mut transaction);
            f(&mut wrapper)
        }?;
        transaction.commit().unwrap();
        Ok(result)
    })
    .await
    .unwrap();
    WebResult::from(&result)
}