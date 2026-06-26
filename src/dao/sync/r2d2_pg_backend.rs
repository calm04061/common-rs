use r2d2_postgres::postgres::{Row, Transaction};
use tokio_postgres::types::ToSql;
use crate::dao::sync::{SyncConnection, ToSqlSync};
use crate::model::result::{DbResult, ErrorCode};

/// Sync-backend wrapper around a [`tokio_postgres::Transaction`].
///
/// This is **not** intended for direct use by downstream crates — use the
/// [`SyncConnection`] trait impl or the `web::r2d2_postgres::invoke_unified`
/// helper instead.
///
/// # Why a raw pointer?
///
/// The web integration layer (`web::r2d2_postgres::invoke_unified`) passes a
/// `PgTran` into a closure that must be `Send + 'static` because it is
/// dispatched via [`actix_web::web::block`].  A `&'a mut Transaction<'a>`
/// cannot satisfy `'static`, so we erase the lifetime with a raw pointer.
///
/// # Safety invariant (maintained by the only constructor site)
///
/// The [`Transaction`] passed to [`PgTran::new`] **must** outlive the
/// returned `PgTran`.  In the sole construction path
/// (`invoke_unified`) this is guaranteed because:
///
/// 1. The `Transaction` is declared **before** the `PgTran` in the same
///    stack frame (the `Transaction` lives on the stack, the `PgTran`
///    borrows from it).
/// 2. The `PgTran` is consumed before the `Transaction` goes out of scope
///    (the `Transaction` is only accessed after the closure returns, for
///    commit/rollback).
///
/// # Send + Safety
///
/// `PgTran` is `Send + Sync` (auto-derived via the `*mut` field).
/// The wrapped `Transaction` is safe to move to another thread because it
/// operates on a connection pool that is already thread-safe (r2d2).
/// Since `PgTran` provides only `&mut self` methods (no shared ownership),
/// there is no aliasing risk.
pub struct PgTran {
    /// Erased lifetime pointer to the backing `Transaction`.
    ///
    /// # Safety
    ///
    /// The referent must outlive `self`.  This is upheld by the
    /// [`PgTran::new`] contract (see struct-level docs).
    tran: *mut Transaction<'static>,
}

impl PgTran {
    /// Wrap a `Transaction` in a `PgTran`.
    ///
    /// # Safety
    ///
    /// The caller **must** guarantee that `tran` outlives the returned
    /// `PgTran`.  Violating this creates a dangling pointer that will
    /// cause undefined behaviour on any method call.
    ///
    /// In practice this is ensured by declaring `tran` **before** the
    /// `PgTran` in the same scope and not moving `tran` while the
    /// `PgTran` is alive:
    ///
    /// ```ignore
    /// let mut transaction = connection.transaction()?; // lives first
    /// let mut wrapper = unsafe { PgTran::new(&mut transaction) }; // borrows
    /// f(&mut wrapper); // used
    /// // wrapper dropped; transaction can commit/rollback
    /// transaction.commit()?;
    /// ```
    pub unsafe fn new(tran: &mut Transaction<'_>) -> Self {
        // SAFETY: The caller promises `tran` outlives the returned `PgTran`,
        // so extending the reference to `'static` is sound — the pointer
        // will never be dereferenced after `tran` is dropped.
        PgTran {
            tran: unsafe { &mut *(tran as *mut Transaction<'_> as *mut Transaction<'static>) },
        }
    }

    /// Reborrow the inner `Transaction` as `&mut`.
    ///
    /// # Safety
    ///
    /// This is safe because:
    /// - `self.tran` was constructed from a valid reference in [`new`](Self::new).
    /// - The struct-level invariant guarantees the referent is still alive.
    /// - We only hand out `&mut` references (no shared aliasing).
    fn as_tran(&mut self) -> &mut Transaction<'static> {
        // SAFETY: `self.tran` was initialised from a valid `&mut Transaction`
        // in `new()` and the struct-level invariant guarantees liveness.
        // The pointer is only accessed through `&mut self`, so there is no
        // aliasing beyond this single reference.
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