use actix_web::web::Data;
use sqlx::{Postgres, Transaction};
use crate::dao::sqlx_postgres::DbPool;
use crate::model::result::{DbResult, WebResult};

pub async fn invoke<F, R>(pool: Data<DbPool>, f: F) -> WebResult<R>
    where
        F: FnOnce(&mut Transaction<'_, Postgres>) -> DbResult<R> + Send + 'static,
        R: Send + 'static + Clone,
{
    let mut transaction = pool.begin().await.unwrap();
    // let mut connection = connection.get().unwrap();
    let result = f(&mut transaction);
    match result {
        Ok(e) => {
            transaction.commit().await.unwrap();
            WebResult::success(e)
        }
        Err(e) => {
            transaction.rollback().await.unwrap();
            WebResult::fail(1, format!("{}", e).as_str())
        }
    }
}

pub async fn after_transaction<T>(result: DbResult<T>, tran: Transaction<'_, Postgres>) -> DbResult<T> {
    return match result {
        Ok(t) => {
            tran.commit().await.unwrap();
            Ok(t)
        }
        Err(_) => {
            tran.rollback().await.unwrap();
            Err(result.err().unwrap())
        }
    };
}