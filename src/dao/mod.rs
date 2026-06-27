/// Unified async DAO — available when at least one sqlx backend is enabled.
#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
pub mod async_dao;

#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
pub use async_dao::AsyncSimpleDao;
#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
use crate::model::result::DbResult;

#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
pub fn get_first<T: for<'r> From<&'r async_dao::DbRow>>(result: DbResult<Vec<async_dao::DbRow>>) -> DbResult<Option<T>> {
    if let Err(e) = result {
        return Err(e);
    }
    let rows = result?;
    Ok(rows.first().map(|r| T::from(r)))
}

#[cfg(any(feature = "sqlx_pg", feature = "sqlx_mysql", feature = "sqlx_oracle"))]
pub fn convert<T: for<'r> From<&'r async_dao::DbRow>>(result: DbResult<Vec<async_dao::DbRow>>) -> DbResult<Vec<T>> {
    if let Err(e) = result {
        return Err(e);
    }
    Ok(result?.iter().map(|r| T::from(r)).collect())
}