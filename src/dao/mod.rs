use crate::model::result::{DbResult, PageRequest, PageResult};

#[cfg(feature = "r2d2_pg")]
pub mod r2d2_postgres;
#[cfg(feature = "sqlx_pg")]
pub mod sqlx_postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub trait SimpleDao<E, T> {
    fn table_name() -> String;

    fn page(page_request: &PageRequest<E>, conn: &mut T) -> DbResult<PageResult<E>>;

    fn list(tran: &mut T) -> DbResult<Vec<E>>;

    fn detail(id: i32, tran: T) -> DbResult<Option<E>>;

    fn delete(id: i32, conn: &mut T) -> DbResult<u64>;
}
