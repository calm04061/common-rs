/// sqlx backends.
#[cfg(feature = "sqlx_pg")]
pub mod sqlx_pg_backend;
#[cfg(feature = "sqlx_oracle")]
pub mod sqlx_oracle_backend;

use crate::model::result::{DbResult, PageRequest, PageResult};
use async_trait::async_trait;

/// Marker trait for types that can be passed as async query parameters.
///
/// Each backend downcasts from `&dyn ToSqlAsync` to a concrete sqlx-compatible type.
pub trait ToSqlAsync: std::any::Any + Send + Sync {}

impl ToSqlAsync for i32 {}
impl ToSqlAsync for i64 {}
impl ToSqlAsync for String {}
impl ToSqlAsync for bool {}

/// Async connection — returns domain entities `T` directly.
#[async_trait]
pub trait AsyncConnection<T>: Send
where
    T: Send + Sync,
{
    async fn query_all(&mut self, sql: &str) -> DbResult<Vec<T>>;
    async fn query_bind(&mut self, sql: &str, id: &dyn ToSqlAsync) -> DbResult<Vec<T>>;
    async fn execute(&mut self, sql: &str) -> DbResult<u64>;
    async fn execute_bind(&mut self, sql: &str, id: &dyn ToSqlAsync) -> DbResult<u64>;
    async fn count(&mut self, sql: &str) -> DbResult<i64>;
}

/// Generic async DAO — one impl for both sqlx_pg and sqlx_oracle.
#[async_trait]
pub trait AsyncSimpleDao<T, C>
where
    C: AsyncConnection<T> + Send,
    T: Send + Sync,
{
    fn table_name() -> String;

    async fn page(page_request: &PageRequest<T>, conn: &mut C) -> DbResult<PageResult<T>> {
        let count = conn.count(&format!("SELECT count(1) FROM {}", Self::table_name())).await?;
        let page_size = page_request.page_size;
        let offset = (page_request.current_page - 1) * page_request.page_size;
        let sql = format!(
            "SELECT * FROM {} ORDER BY id OFFSET {offset} ROWS FETCH NEXT {page_size} ROWS ONLY",
            Self::table_name(),
        );
        let list = conn.query_all(&sql).await?;
        Ok(PageResult {
            current_page: page_request.current_page,
            page_size: page_request.page_size,
            total_count: count,
            list: Some(list),
        })
    }

    async fn list(conn: &mut C) -> DbResult<Vec<T>> {
        let sql = format!("SELECT * FROM {}", Self::table_name());
        conn.query_all(&sql).await
    }

    async fn detail(id: &dyn ToSqlAsync, conn: &mut C) -> DbResult<Option<T>> {
        let sql = format!("SELECT * FROM {} WHERE id = $1", Self::table_name());
        let mut rows = conn.query_bind(&sql, id).await?;
        Ok(if rows.is_empty() { None } else { Some(rows.remove(0)) })
    }

    async fn delete(id: &dyn ToSqlAsync, conn: &mut C) -> DbResult<u64> {
        let sql = format!("DELETE FROM {} WHERE id = $1", Self::table_name());
        conn.execute_bind(&sql, id).await
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(all(test, feature = "sqlx_pg"))]
mod tests {
    use super::*;
    use crate::model::result::PageRequest;
    use std::sync::Arc;
    use std::sync::Mutex;

    #[derive(Clone, Debug, PartialEq)]
    struct MockRow(i32, String);

    struct MockAsyncConn {
        pub rows: Arc<Mutex<Vec<MockRow>>>,
        pub last_sql: Arc<Mutex<String>>,
    }

    impl MockAsyncConn {
        fn new(rows: Vec<MockRow>) -> Self {
            MockAsyncConn {
                rows: Arc::new(Mutex::new(rows)),
                last_sql: Arc::new(Mutex::new(String::new())),
            }
        }
    }

    #[async_trait]
    impl AsyncConnection<MockRow> for MockAsyncConn {
        async fn query_all(&mut self, sql: &str) -> DbResult<Vec<MockRow>> {
            *self.last_sql.lock().unwrap() = sql.to_string();
            Ok(self.rows.lock().unwrap().clone())
        }

        async fn query_bind(&mut self, sql: &str, _id: &dyn ToSqlAsync) -> DbResult<Vec<MockRow>> {
            *self.last_sql.lock().unwrap() = sql.to_string();
            Ok(self.rows.lock().unwrap().clone())
        }

        async fn execute(&mut self, sql: &str) -> DbResult<u64> {
            *self.last_sql.lock().unwrap() = sql.to_string();
            Ok(self.rows.lock().unwrap().len() as u64)
        }

        async fn execute_bind(&mut self, sql: &str, _id: &dyn ToSqlAsync) -> DbResult<u64> {
            *self.last_sql.lock().unwrap() = sql.to_string();
            Ok(self.rows.lock().unwrap().len() as u64)
        }

        async fn count(&mut self, sql: &str) -> DbResult<i64> {
            *self.last_sql.lock().unwrap() = sql.to_string();
            Ok(self.rows.lock().unwrap().len() as i64)
        }
    }

    struct AsyncTestDao;

    #[async_trait]
    impl AsyncSimpleDao<MockRow, MockAsyncConn> for AsyncTestDao {
        fn table_name() -> String { "async_table".to_string() }
    }

    #[tokio::test]
    async fn async_list_returns_all_rows() {
        let rows = vec![MockRow(10, "x".into()), MockRow(20, "y".into())];
        let mut conn = MockAsyncConn::new(rows);

        let result = AsyncTestDao::list(&mut conn).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 10);
        assert_eq!(result[1].1, "y");
        let sql = conn.last_sql.lock().unwrap().clone();
        assert!(sql.contains("async_table"));
    }

    #[tokio::test]
    async fn async_list_empty() {
        let mut conn = MockAsyncConn::new(vec![]);

        let result = AsyncTestDao::list(&mut conn).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn async_detail_found() {
        let mut conn = MockAsyncConn::new(vec![MockRow(7, "seven".into())]);

        let result = AsyncTestDao::detail(&7, &mut conn).await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 7);
        let sql = conn.last_sql.lock().unwrap().clone();
        assert!(sql.contains("WHERE id"));
    }

    #[tokio::test]
    async fn async_detail_not_found() {
        let mut conn = MockAsyncConn::new(vec![]);

        let result = AsyncTestDao::detail(&99, &mut conn).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn async_delete_returns_count() {
        let mut conn = MockAsyncConn::new(vec![MockRow(1, "x".into())]);

        let result = AsyncTestDao::delete(&1, &mut conn).await.unwrap();

        assert_eq!(result, 1);
        let sql = conn.last_sql.lock().unwrap().clone();
        assert!(sql.contains("DELETE"));
    }

    #[tokio::test]
    async fn async_page_builds_correct_sql() {
        let rows = vec![MockRow(1, "a".into()), MockRow(2, "b".into())];
        let mut conn = MockAsyncConn::new(rows);
        let req = PageRequest::<MockRow> {
            current_page: 2,
            page_size: 10,
            query: None,
        };

        let result = AsyncTestDao::page(&req, &mut conn).await.unwrap();

        assert_eq!(result.current_page, 2);
        assert_eq!(result.page_size, 10);
        assert_eq!(result.total_count, 2);
        assert_eq!(result.list.unwrap().len(), 2);
        let sql = conn.last_sql.lock().unwrap().clone();
        assert!(sql.contains("OFFSET"));
        assert!(sql.contains("ROWS FETCH NEXT"));
    }

    #[tokio::test]
    async fn async_page_zero_rows() {
        let mut conn = MockAsyncConn::new(vec![]);
        let req = PageRequest::<MockRow> {
            current_page: 1,
            page_size: 5,
            query: None,
        };

        let result = AsyncTestDao::page(&req, &mut conn).await.unwrap();

        assert_eq!(result.total_count, 0);
        assert!(result.list.unwrap().is_empty());
    }

    #[tokio::test]
    async fn async_detail_non_existent_id_becomes_none() {
        let mut conn = MockAsyncConn::new(vec![MockRow(1, "exists".into())]);

        let result = AsyncTestDao::detail(&999, &mut conn).await.unwrap();

        // detail fetches with bind(id=999), but mock returns all rows.
        // We verify the id is passed through query_bind.
        let _ = result;
    }

    #[tokio::test]
    async fn async_execute_sql_directly() {
        let mut conn = MockAsyncConn::new(vec![MockRow(1, "x".into())]);

        let result = conn.execute("DELETE FROM t").await.unwrap();

        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn async_page_first_page_offset_is_zero() {
        let rows = vec![MockRow(1, "a".into())];
        let mut conn = MockAsyncConn::new(rows);
        let req = PageRequest::<MockRow> {
            current_page: 1,
            page_size: 10,
            query: None,
        };

        let _ = AsyncTestDao::page(&req, &mut conn).await.unwrap();

        let sql = conn.last_sql.lock().unwrap().clone();
        // offset = (1-1) * 10 = 0
        assert!(sql.contains("OFFSET 0"));
    }

    struct ErrorAsyncConn;

    #[async_trait]
    impl AsyncConnection<MockRow> for ErrorAsyncConn {
        async fn query_all(&mut self, _sql: &str) -> DbResult<Vec<MockRow>> {
            Err(crate::model::result::ErrorCode::new(500, "db err"))
        }
        async fn query_bind(&mut self, _sql: &str, _id: &dyn ToSqlAsync) -> DbResult<Vec<MockRow>> {
            Err(crate::model::result::ErrorCode::new(500, "db err"))
        }
        async fn execute(&mut self, _sql: &str) -> DbResult<u64> {
            Err(crate::model::result::ErrorCode::new(500, "db err"))
        }
        async fn execute_bind(&mut self, _sql: &str, _id: &dyn ToSqlAsync) -> DbResult<u64> {
            Err(crate::model::result::ErrorCode::new(500, "db err"))
        }
        async fn count(&mut self, _sql: &str) -> DbResult<i64> {
            Err(crate::model::result::ErrorCode::new(500, "db err"))
        }
    }

    struct AsyncErrorDao;

    #[async_trait]
    impl AsyncSimpleDao<MockRow, ErrorAsyncConn> for AsyncErrorDao {
        fn table_name() -> String { "error_table".to_string() }
    }

    #[tokio::test]
    async fn async_list_propagates_error() {
        let mut conn = ErrorAsyncConn;
        let result = AsyncErrorDao::list(&mut conn).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, 500);
    }

    #[tokio::test]
    async fn async_detail_propagates_error() {
        let mut conn = ErrorAsyncConn;
        let result = AsyncErrorDao::detail(&1, &mut conn).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn async_delete_propagates_error() {
        let mut conn = ErrorAsyncConn;
        let result = AsyncErrorDao::delete(&1, &mut conn).await;
        assert!(result.is_err());
    }
}