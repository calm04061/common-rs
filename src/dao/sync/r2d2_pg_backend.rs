use r2d2_postgres::postgres::{Row, Transaction};
use tokio_postgres::types::ToSql;
use crate::dao::sync::{SyncConnection, ToSqlSync};
use crate::model::result::{DbResult, ErrorCode};

/// Internal backend wrapper. Not intended for direct use by downstream crates.
///
/// Uses a raw pointer internally to sidestep lifetime complexity
/// with nested mutable borrows in the web helper layer.
pub struct PgTran {
    tran: *mut Transaction<'static>,
}

impl PgTran {
    /// # Safety
    ///
    /// `tran` must outlive the returned `PgTran`.
    pub unsafe fn new(tran: &mut Transaction<'_>) -> Self {
        PgTran {
            tran: unsafe { &mut *(tran as *mut Transaction<'_> as *mut Transaction<'static>) },
        }
    }

    fn as_tran(&mut self) -> &mut Transaction<'static> {
        unsafe { &mut *self.tran }
    }
}

impl<T> SyncConnection<T> for PgTran
where
    T: From<Row>,
{
    fn query_all(&mut self, sql: &str) -> DbResult<Vec<T>> {
        let rows = self.as_tran().query(sql, &[]).map_err(to_ec)?;
        Ok(rows.into_iter().map(T::from).collect())
    }

    fn query_bind(&mut self, sql: &str, id: &dyn ToSqlSync) -> DbResult<Vec<T>> {
        with_slice(id, |slice| {
            let rows = self.as_tran().query(sql, slice).map_err(to_ec)?;
            Ok(rows.into_iter().map(T::from).collect())
        })
    }

    fn execute(&mut self, sql: &str) -> DbResult<u64> {
        self.as_tran().execute(sql, &[]).map_err(to_ec)
    }

    fn execute_bind(&mut self, sql: &str, id: &dyn ToSqlSync) -> DbResult<u64> {
        with_slice(id, |slice| self.as_tran().execute(sql, slice).map_err(to_ec))
    }

    fn count(&mut self, sql: &str) -> DbResult<i64> {
        let row = self.as_tran().query_one(sql, &[]).map_err(to_ec)?;
        Ok(row.get::<_, i64>(0))
    }
}

fn to_ec(e: r2d2_postgres::postgres::Error) -> ErrorCode {
    ErrorCode::new(1, &e.to_string())
}

fn with_slice<F, T>(id: &dyn ToSqlSync, f: F) -> DbResult<T>
where
    F: FnOnce(&[&(dyn ToSql + Sync)]) -> DbResult<T>,
{
    let any = id as &dyn std::any::Any;
    if let Some(v) = any.downcast_ref::<i32>() {
        f(&[v as &(dyn ToSql + Sync)])
    } else if let Some(v) = any.downcast_ref::<i64>() {
        f(&[v as &(dyn ToSql + Sync)])
    } else if let Some(v) = any.downcast_ref::<String>() {
        f(&[v as &(dyn ToSql + Sync)])
    } else if let Some(v) = any.downcast_ref::<bool>() {
        f(&[v as &(dyn ToSql + Sync)])
    } else {
        Err(ErrorCode::new(1, "unsupported bind type for r2d2_pg"))
    }
}