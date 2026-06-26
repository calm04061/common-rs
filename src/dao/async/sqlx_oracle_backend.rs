use async_trait::async_trait;
use sqlx::{query, Executor, Row, Transaction};
use sqlx::AssertSqlSafe;
use sqlx_oracle::Oracle;
use crate::dao::r#async::AsyncConnection;
use crate::model::result::{DbResult, ErrorCode};

type OracleRow = <Oracle as sqlx::Database>::Row;

/// Wrapper around sqlx Oracle Transaction.
pub struct OracleAsyncTran<'a>(pub &'a mut Transaction<'a, Oracle>);

fn to_ec(e: sqlx::Error) -> ErrorCode {
    ErrorCode::new(1, &e.to_string())
}

#[async_trait]
impl<T> AsyncConnection<T> for OracleAsyncTran<'_>
where
    T: for<'r> From<&'r OracleRow> + Send + Sync,
{
    async fn query_all(&mut self, sql: &str) -> DbResult<Vec<T>> {
        let rows = self
            .0
            .fetch_all(query(AssertSqlSafe(sql.to_owned())))
            .await
            .map_err(to_ec)?;
        Ok(rows.iter().map(|r| T::from(r)).collect())
    }

    async fn query_bind(&mut self, sql: &str, id: i32) -> DbResult<Vec<T>> {
        let rows = self
            .0
            .fetch_all(query(AssertSqlSafe(sql.to_owned())).bind(id))
            .await
            .map_err(to_ec)?;
        Ok(rows.iter().map(|r| T::from(r)).collect())
    }

    async fn execute(&mut self, sql: &str) -> DbResult<u64> {
        Ok(self
            .0
            .execute(query(AssertSqlSafe(sql.to_owned())))
            .await
            .map_err(to_ec)?
            .rows_affected())
    }

    async fn execute_bind(&mut self, sql: &str, id: i32) -> DbResult<u64> {
        Ok(self
            .0
            .execute(query(AssertSqlSafe(sql.to_owned())).bind(id))
            .await
            .map_err(to_ec)?
            .rows_affected())
    }

    async fn count(&mut self, sql: &str) -> DbResult<i64> {
        let rows = self
            .0
            .fetch_all(query(AssertSqlSafe(sql.to_owned())))
            .await
            .map_err(to_ec)?;
        let count: i64 = rows[0].get(0);
        Ok(count)
    }
}
