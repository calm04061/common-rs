use std::ops::{Deref, DerefMut};
use async_trait::async_trait;
use sqlx::postgres::{PgRow, Postgres};
use sqlx::{query, Executor, Pool, Row, Transaction};
use sqlx::AssertSqlSafe;
use crate::dao::async_dao::{AsyncConnection, ToSqlAsync};
use crate::model::result::{DbResult, ErrorCode};

/// Wrapper around sqlx Postgres Transaction.
///
/// Owns the Transaction so controllers never see `Postgres` or `Transaction` directly.
pub struct PgAsyncTran<'c>(pub Transaction<'c, Postgres>);

impl<'c> Deref for PgAsyncTran<'c> {
    type Target = Transaction<'c, Postgres>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'c> DerefMut for PgAsyncTran<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'c> PgAsyncTran<'c> {
    /// Begin a new transaction from a connection pool.
    pub async fn begin(pool: &Pool<Postgres>) -> Result<Self, ErrorCode> {
        pool.begin().await.map(Self).map_err(|e| ErrorCode::new(1, &e.to_string()))
    }

    /// Commit the transaction.
    pub async fn commit(self) -> Result<(), ErrorCode> {
        self.0.commit().await.map_err(|e| ErrorCode::new(1, &e.to_string()))
    }

    /// Roll back the transaction.
    pub async fn rollback(self) -> Result<(), ErrorCode> {
        self.0.rollback().await.map_err(|e| ErrorCode::new(1, &e.to_string()))
    }
}

fn to_ec(e: sqlx::Error) -> ErrorCode {
    ErrorCode::new(1, &e.to_string())
}

#[async_trait]
impl<'c, T> AsyncConnection<T> for PgAsyncTran<'c>
where
    T: From<PgRow> + Send + Sync,
{
    async fn query_all(&mut self, sql: &str) -> DbResult<Vec<T>> {
        let rows = self
            .0
            .fetch_all(query(AssertSqlSafe(sql.to_owned())))
            .await
            .map_err(to_ec)?;
        Ok(rows.into_iter().map(|r| T::from(r)).collect())
    }

    async fn query_bind(&mut self, sql: &str, id: &dyn ToSqlAsync) -> DbResult<Vec<T>> {
        let q = query(AssertSqlSafe(sql.to_owned()));
        let q = crate::bind_async_param!(id, q, "sqlx_pg");
        let rows = self.0.fetch_all(q).await.map_err(to_ec)?;
        Ok(rows.into_iter().map(|r| T::from(r)).collect())
    }

    async fn execute(&mut self, sql: &str) -> DbResult<u64> {
        Ok(self
            .0
            .execute(query(AssertSqlSafe(sql.to_owned())))
            .await
            .map_err(to_ec)?
            .rows_affected())
    }

    async fn execute_bind(&mut self, sql: &str, id: &dyn ToSqlAsync) -> DbResult<u64> {
        let q = query(AssertSqlSafe(sql.to_owned()));
        let q = crate::bind_async_param!(id, q, "sqlx_pg");
        Ok(self.0.execute(q).await.map_err(to_ec)?.rows_affected())
    }

    async fn count(&mut self, sql: &str) -> DbResult<i64> {
        let row = self
            .0
            .fetch_one(query(AssertSqlSafe(sql.to_owned())))
            .await
            .map_err(to_ec)?;
        Ok(row.get::<i64, _>(0))
    }
}
