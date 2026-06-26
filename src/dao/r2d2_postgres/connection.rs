use r2d2::{Pool};
use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;
use tokio_postgres::Error;

pub type DbPool = Pool<PostgresConnectionManager<NoTls>>;
pub type DbResult<T> = Result<T, Error>;
