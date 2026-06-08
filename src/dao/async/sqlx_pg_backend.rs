use async_trait::async_trait;
use sqlx::postgres::{PgRow, Postgres};
use sqlx::{query, Executor, Row, Transaction};
use sqlx::AssertSqlSafe;
use crate::dao::r#async::AsyncConnection;
use crate::model::result::DbResult;

/// Wrapper around sqlx Postgres Transaction.
pub struct PgAsyncTran<'a>(pub &'a mut Transaction<'a, Postgres>);

#[async_trait]
impl<T> AsyncConnection<T> for PgAsyncTran<'_>
where
    T: From<PgRow> + Send + Sync,
{
    async fn query_all(&mut self, sql: &str) -> DbResult<Vec<T>> {
        let rows = self
            .0
            .fetch_all(query(AssertSqlSafe(sql.to_owned())))
            .await
            .unwrap();
        Ok(rows.into_iter().map(|r| T::from(r)).collect())
    }

    async fn query_bind(&mut self, sql: &str, id: i32) -> DbResult<Vec<T>> {
        let rows = self
            .0
            .fetch_all(query(AssertSqlSafe(sql.to_owned())).bind(id))
            .await
            .unwrap();
        Ok(rows.into_iter().map(|r| T::from(r)).collect())
    }

    async fn execute(&mut self, sql: &str) -> DbResult<u64> {
        Ok(self
            .0
            .execute(query(AssertSqlSafe(sql.to_owned())))
            .await
            .unwrap()
            .rows_affected())
    }

    async fn execute_bind(&mut self, sql: &str, id: i32) -> DbResult<u64> {
        Ok(self
            .0
            .execute(query(AssertSqlSafe(sql.to_owned())).bind(id))
            .await
            .unwrap()
            .rows_affected())
    }

    async fn count(&mut self, sql: &str) -> DbResult<i64> {
        let row = self
            .0
            .fetch_one(query(AssertSqlSafe(sql.to_owned())))
            .await
            .unwrap();
        Ok(row.get::<i64, _>(0))
    }
}