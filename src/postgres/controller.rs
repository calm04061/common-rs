use actix_web::web;
use actix_web::web::Data;
use r2d2_postgres::postgres::Transaction;
use crate::model::result::WebResult;
use crate::postgres::connection::{DbPool, DbResult};

pub fn invoke<F, R>(pool: &DbPool, f: F) -> WebResult<R>
    where
        F: FnOnce(&mut Transaction) -> DbResult<R> + Send + 'static,
        R: Send + 'static + Clone,
{
    let mut connection = pool.get().unwrap();
    // let mut connection = connection.get().unwrap();
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
//
// pub async fn invoke_block<F, R>(f: F) -> WebResult<R>
//     where
//         F: FnOnce(&mut Transaction) -> DbResult<R> + Send + 'static,
//         R: Send + 'static + Clone {
//     web::block(move || {
//         invoke(f)
//     }).await.unwrap()
// }

pub async fn invoke_block<F, R>(pool: Data<DbPool>, f: F) -> WebResult<R>
    where
        F: FnOnce(&mut Transaction) -> DbResult<R> + Send + 'static,
        R: Send + 'static + Clone {
    web::block(move || {
        invoke(&pool, f)
    }).await.unwrap()
}
