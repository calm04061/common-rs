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

/// 查询结果（DML）。
///
/// 记录受影响的记录行数。
#[derive(Debug, Default)]
pub struct QueryResult {
    pub rows_affected: u64,
}

impl QueryResult {
    /// 返回 DML 语句影响的行数。
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<QueryResult> for QueryResult {
    /// 累加多个查询结果的影响行数。
    fn extend<T: IntoIterator<Item = QueryResult>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
        }
    }
}
