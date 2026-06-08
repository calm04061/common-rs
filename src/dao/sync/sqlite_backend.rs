use r2d2_sqlite::rusqlite::{Row, Transaction};
use crate::dao::sync::{SyncConnection, ToSqlSync};
use crate::model::result::{DbResult, ErrorCode};

type SqliteToSql = dyn r2d2_sqlite::rusqlite::types::ToSql;

/// Wrapper around sqlite Transaction.
pub struct SqliteTran<'a>(pub &'a mut Transaction<'a>);

impl<'a, T> SyncConnection<T> for SqliteTran<'a>
where
    T: for<'row> From<&'row Row<'row>>,
{
    fn query_all(&mut self, sql: &str) -> DbResult<Vec<T>> {
        let mut stmt = self.0.prepare(sql).map_err(to_ec)?;
        let mut rows = stmt.query([]).map_err(to_ec)?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().map_err(to_ec)? {
            out.push(T::from(row));
        }
        Ok(out)
    }

    fn query_bind(&mut self, sql: &str, id: &dyn ToSqlSync) -> DbResult<Vec<T>> {
        let mut out = Vec::new();
        with_slice(id, |params| {
            let mut stmt = self.0.prepare(sql).map_err(to_ec)?;
            let mut rows = stmt.query(params).map_err(to_ec)?;
            while let Some(row) = rows.next().map_err(to_ec)? {
                out.push(T::from(row));
            }
            Ok(())
        })?;
        Ok(out)
    }

    fn execute(&mut self, sql: &str) -> DbResult<u64> {
        self.0.execute(sql, []).map(|n| n as u64).map_err(to_ec)
    }

    fn execute_bind(&mut self, sql: &str, id: &dyn ToSqlSync) -> DbResult<u64> {
        with_slice(id, |params| {
            let mut stmt = self.0.prepare(sql).map_err(to_ec)?;
            stmt.execute(params).map(|n| n as u64).map_err(to_ec)
        })
    }

    fn count(&mut self, sql: &str) -> DbResult<i64> {
        let mut stmt = self.0.prepare(sql).map_err(to_ec)?;
        stmt.query_row([], |row| row.get::<_, i64>(0))
            .map_err(to_ec)
    }
}

fn to_ec(e: r2d2_sqlite::rusqlite::Error) -> ErrorCode {
    ErrorCode::new(1, &e.to_string())
}

fn with_slice<F, T>(id: &dyn ToSqlSync, f: F) -> DbResult<T>
where
    F: FnOnce(&[&SqliteToSql]) -> DbResult<T>,
{
    let any = id as &dyn std::any::Any;
    if let Some(v) = any.downcast_ref::<i32>() {
        f(&[v as &SqliteToSql])
    } else if let Some(v) = any.downcast_ref::<i64>() {
        f(&[v as &SqliteToSql])
    } else if let Some(v) = any.downcast_ref::<String>() {
        f(&[v as &SqliteToSql])
    } else {
        Err(ErrorCode::new(1, "unsupported bind type for sqlite"))
    }
}