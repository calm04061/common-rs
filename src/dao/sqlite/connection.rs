use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::database::CommonConnectionHolder;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbResult<T> = r2d2_sqlite::rusqlite::Result<T>;

lazy_static! {
    static ref CONNECTION_HOLDER: Mutex<ConnectionHolder> = Mutex::new(ConnectionHolder::new());
}
struct ConnectionHolder {
    object_ref: Option<Arc<Mutex<DbPool>>>,
}
impl ConnectionHolder {
    fn new() -> Self {
        ConnectionHolder { object_ref: None }
    }

}

impl CommonConnectionHolder<DbPool> for ConnectionHolder{
    fn get_object_ref(&self) -> Option<Arc<Mutex<DbPool>>> {
        self.object_ref.clone()
    }

    fn set_object_ref(&mut self, obj_ref: Option<Arc<Mutex<DbPool>>>)  {
        self.object_ref = obj_ref;
    }

    fn init_connection() -> DbPool {
        let database_url = dotenv::var("DATABASE_URL").unwrap_or(String::from("/var/lib/cloud/config/file.db"));

        let connection_manager = SqliteConnectionManager::file(database_url);
        let pool = Pool::builder()
            .build(connection_manager)
            .expect("Failed to create pool.");

        return pool;
    }
}
#[cfg(feature = "sqlite")]
pub fn get_connection() -> Arc<Mutex<DbPool>> {
    let mut guard = CONNECTION_HOLDER.lock().unwrap();
    guard.get()
}
