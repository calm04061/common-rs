use actix_web::web::Data;
use sqlx::{Postgres, Transaction};
use crate::dao::sqlx_postgres::DbPool;
use crate::model::result::{DbResult, ErrorCode, WebResult};

pub async fn invoke<F, R>(pool: Data<DbPool>, f: F) -> WebResult<R>
    where
        F: FnOnce(&mut Transaction<'_, Postgres>) -> DbResult<R> + Send + 'static,
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

pub async fn after_transaction<T>(result: DbResult<T>, tran: Transaction<'_, Postgres>) -> DbResult<T> {
    match result {
        Ok(t) => {
            tran.commit().await.map_err(|e| ErrorCode::new(1, &e.to_string()))?;
            Ok(t)
        }
        Err(e) => {
            let _ = tran.rollback().await;
            Err(e)
        }
    }
}
