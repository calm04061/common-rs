// use std::process::exit;
// use std::sync::{Arc, Mutex};
// use lazy_static::lazy_static;
// use log::{error};
use r2d2::{Pool};
use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;
use tokio_postgres::Error;
// use crate::database::CommonConnectionHolder;
//
pub type DbPool = Pool<PostgresConnectionManager<NoTls>>;
pub type DbResult<T> = Result<T, Error>;
//
// lazy_static! {
//     static ref CONNECTION_HOLDER: Mutex<ConnectionHolder> = Mutex::new(ConnectionHolder::new());
// }
// struct ConnectionHolder {
//     object_ref: Option<Arc<Mutex<DbPool>>>,
// }
//
// impl ConnectionHolder {
//     fn new() -> Self {
//         ConnectionHolder { object_ref: None }
//     }
// }
//
// impl CommonConnectionHolder<DbPool> for ConnectionHolder{
//     fn get_object_ref(&self) -> Option<Arc<Mutex<DbPool>>> {
//         self.object_ref.clone()
//     }
//
//     fn set_object_ref(&mut self, obj_ref: Option<Arc<Mutex<DbPool>>>)  {
//         self.object_ref = obj_ref;
//     }
//
//     fn init_connection() -> DbPool {
//         let database_url = dotenv::var("DATABASE_URL").unwrap_or(String::from("r2d2_postgres://r2d2_postgres:r2d2_postgres@127.0.0.1:5432/manager"));
//
//         let postgres_manager = PostgresConnectionManager::new(database_url.parse().unwrap(),
//                                                               NoTls,
//         );
//         let postgres_pool = Pool::new(postgres_manager);
//         let postgres_pool = match postgres_pool {
//             Ok(pool) => {
//                 pool
//             }
//             Err(e) => {
//                 error!("{}",e);
//                 exit(1);
//             }
//         };
//         return postgres_pool;
//     }
// }
//
// impl Drop for ConnectionHolder {
//     fn drop(&mut self) {
//         match &self.object_ref {
//             None => {}
//             Some(e) => {
//                 let result = e.lock();
//                 let connection = result.unwrap();
//                 let connection = connection.get().unwrap();
//                 drop(connection);
//             }
//         }
//     }
// }
//
//
//
// // #[cfg(feature = "r2d2_postgres")]
// // pub fn get_connection() -> Arc<Mutex<DbPool>> {
// //     let mut guard = CONNECTION_HOLDER.lock().unwrap();
// //     guard.get()
// // }
