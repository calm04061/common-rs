/// Sync backend implementations.
#[cfg(feature = "r2d2_pg")]
pub mod r2d2_pg_backend;
#[cfg(feature = "sqlite")]
pub mod sqlite_backend;

use crate::model::result::{DbResult, PageRequest, PageResult};

/// Marker trait for types that can be passed as query parameters.
///
/// Each backend downcasts from `&dyn ToSqlSync` to a concrete type.
pub trait ToSqlSync: std::any::Any + Send + Sync {}

impl ToSqlSync for i32 {}
impl ToSqlSync for i64 {}
impl ToSqlSync for String {}
impl ToSqlSync for bool {}

/// Sync connection — returns domain entities (`T`) directly.
///
/// Conversion from the backend's Row to `T` happens inside the impl.
pub trait SyncConnection<T> {
    fn query_all(&mut self, sql: &str) -> DbResult<Vec<T>>;
    fn query_bind(&mut self, sql: &str, id: &dyn ToSqlSync) -> DbResult<Vec<T>>;
    fn execute(&mut self, sql: &str) -> DbResult<u64>;
    fn execute_bind(&mut self, sql: &str, id: &dyn ToSqlSync) -> DbResult<u64>;
    fn count(&mut self, sql: &str) -> DbResult<i64>;
}

/// Generic sync DAO — one impl works for both r2d2_pg and sqlite.
pub trait SimpleDao<T, C>
where
    C: SyncConnection<T>,
{
    fn table_name() -> String;

    fn page(page_request: &PageRequest<T>, conn: &mut C) -> DbResult<PageResult<T>> {
        let count = conn.count(&format!("SELECT count(1) FROM {}", Self::table_name()))?;
        let page_size = page_request.page_size;
        let offset = (page_request.current_page - 1) * page_request.page_size;
        let sql = format!(
            "SELECT * FROM {} ORDER BY id LIMIT {page_size} OFFSET {offset}",
            Self::table_name(),
        );
        let list = conn.query_all(&sql)?;
        Ok(PageResult {
            current_page: page_request.current_page,
            page_size: page_request.page_size,
            total_count: count,
            list: Some(list),
        })
    }

    fn list(conn: &mut C) -> DbResult<Vec<T>> {
        let sql = format!("SELECT * FROM {}", Self::table_name());
        conn.query_all(&sql)
    }

    fn detail(id: &dyn ToSqlSync, conn: &mut C) -> DbResult<Option<T>> {
        let sql = format!("SELECT * FROM {} WHERE id = $1", Self::table_name());
        let mut rows = conn.query_bind(&sql, id)?;
        Ok(if rows.is_empty() { None } else { Some(rows.remove(0)) })
    }

    fn delete(id: &dyn ToSqlSync, conn: &mut C) -> DbResult<u64> {
        let sql = format!("DELETE FROM {} WHERE id = $1", Self::table_name());
        conn.execute_bind(&sql, id)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::result::{DbResult, ErrorCode, PageRequest};

    // --- Mock connection that records SQL and returns canned data ---

    #[derive(Clone, Debug)]
    struct MockRow(i32, String);

    struct MockConn {
        pub rows: Vec<MockRow>,
        pub last_sql: String,
    }

    impl MockConn {
        fn new(rows: Vec<MockRow>) -> Self {
            MockConn { rows, last_sql: String::new() }
        }
    }

    impl SyncConnection<MockRow> for MockConn {
        fn query_all(&mut self, sql: &str) -> DbResult<Vec<MockRow>> {
            self.last_sql = sql.to_string();
            Ok(self.rows.clone())
        }

        fn query_bind(&mut self, sql: &str, _id: &dyn ToSqlSync) -> DbResult<Vec<MockRow>> {
            self.last_sql = sql.to_string();
            Ok(self.rows.clone())
        }

        fn execute(&mut self, sql: &str) -> DbResult<u64> {
            self.last_sql = sql.to_string();
            Ok(self.rows.len() as u64)
        }

        fn execute_bind(&mut self, sql: &str, _id: &dyn ToSqlSync) -> DbResult<u64> {
            self.last_sql = sql.to_string();
            Ok(self.rows.len() as u64)
        }

        fn count(&mut self, sql: &str) -> DbResult<i64> {
            self.last_sql = sql.to_string();
            Ok(self.rows.len() as i64)
        }
    }

    // --- DAO stub ---

    struct TestDao;

    impl SimpleDao<MockRow, MockConn> for TestDao {
        fn table_name() -> String { "test_table".to_string() }
    }

    // --- Tests ---

    #[test]
    fn sync_list_returns_all_rows() {
        let rows = vec![MockRow(1, "a".into()), MockRow(2, "b".into())];
        let mut conn = MockConn::new(rows.clone());

        let result = TestDao::list(&mut conn).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 1);
        assert_eq!(result[1].1, "b");
        assert!(conn.last_sql.contains("test_table"));
    }

    #[test]
    fn sync_list_empty() {
        let mut conn = MockConn::new(vec![]);

        let result = TestDao::list(&mut conn).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn sync_detail_found() {
        let rows = vec![MockRow(42, "found".into())];
        let mut conn = MockConn::new(rows);

        let result = TestDao::detail(&42, &mut conn).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 42);
        assert!(conn.last_sql.contains("WHERE id"));
    }

    #[test]
    fn sync_detail_not_found() {
        let mut conn = MockConn::new(vec![]);

        let result = TestDao::detail(&99, &mut conn).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn sync_delete_returns_count() {
        let mut conn = MockConn::new(vec![MockRow(1, "x".into())]);

        let result = TestDao::delete(&1, &mut conn).unwrap();

        assert_eq!(result, 1);
        assert!(conn.last_sql.contains("DELETE"));
    }

    #[test]
    fn sync_page_builds_correct_sql() {
        let rows = vec![MockRow(1, "a".into()), MockRow(2, "b".into())];
        let mut conn = MockConn::new(rows);
        let req = PageRequest::<MockRow> {
            current_page: 2,
            page_size: 10,
            query: None,
        };

        let result = TestDao::page(&req, &mut conn).unwrap();

        assert_eq!(result.current_page, 2);
        assert_eq!(result.page_size, 10);
        assert_eq!(result.total_count, 2);
        assert_eq!(result.list.unwrap().len(), 2);
        assert!(conn.last_sql.contains("LIMIT"));
        assert!(conn.last_sql.contains("OFFSET"));
    }

    #[test]
    fn sync_page_zero_rows() {
        let mut conn = MockConn::new(vec![]);
        let req = PageRequest::<MockRow> {
            current_page: 1,
            page_size: 5,
            query: None,
        };

        let result = TestDao::page(&req, &mut conn).unwrap();

        assert_eq!(result.total_count, 0);
        assert_eq!(result.list.unwrap().len(), 0);
    }

    #[test]
    fn to_sql_sync_downcast_i32() {
        let id: &dyn ToSqlSync = &42;
        let any = id as &dyn std::any::Any;
        assert!(any.downcast_ref::<i32>().is_some());
        assert!(any.downcast_ref::<i64>().is_none());
        assert!(any.downcast_ref::<String>().is_none());
    }

    #[test]
    fn to_sql_sync_downcast_string() {
        let s = "hello".to_string();
        let id: &dyn ToSqlSync = &s;
        let any = id as &dyn std::any::Any;
        assert!(any.downcast_ref::<String>().is_some());
        assert!(any.downcast_ref::<i32>().is_none());
    }

    #[test]
    fn to_sql_sync_downcast_bool() {
        let id: &dyn ToSqlSync = &true;
        let any = id as &dyn std::any::Any;
        assert!(any.downcast_ref::<bool>().is_some());
    }

    #[test]
    fn to_sql_sync_downcast_i64() {
        let id: &dyn ToSqlSync = &100i64;
        let any = id as &dyn std::any::Any;
        assert!(any.downcast_ref::<i64>().is_some());
        assert!(any.downcast_ref::<i32>().is_none());
    }

    #[test]
    fn sync_execute_sql_directly() {
        let rows = vec![MockRow(1, "a".into())];
        let mut conn = MockConn::new(rows);

        let result = conn.execute("UPDATE test_table SET x = 1").unwrap();

        assert_eq!(result, 1);
        assert!(conn.last_sql.contains("test_table"));
    }

    #[test]
    fn sync_page_first_page_offset_is_zero() {
        let rows = vec![MockRow(1, "a".into())];
        let mut conn = MockConn::new(rows);
        let req = PageRequest::<MockRow> {
            current_page: 1,
            page_size: 10,
            query: None,
        };

        let _ = TestDao::page(&req, &mut conn).unwrap();

        // offset = (1-1) * 10 = 0
        assert!(conn.last_sql.contains("OFFSET 0"));
    }

    #[test]
    fn sync_detail_with_string_id() {
        let rows = vec![MockRow(1, "found".into())];
        let mut conn = MockConn::new(rows);

        let result = TestDao::detail(&"test-id".to_string(), &mut conn).unwrap();

        assert!(result.is_some());
        assert!(conn.last_sql.contains("WHERE id"));
    }

    #[test]
    fn sync_detail_returns_none_for_empty_result() {
        let mut conn = MockConn::new(vec![]);

        let result = TestDao::detail(&"missing".to_string(), &mut conn).unwrap();

        assert!(result.is_none());
    }

    struct ErrorConn;

    impl SyncConnection<MockRow> for ErrorConn {
        fn query_all(&mut self, _sql: &str) -> DbResult<Vec<MockRow>> {
            Err(ErrorCode::new(500, "db error"))
        }
        fn query_bind(&mut self, _sql: &str, _id: &dyn ToSqlSync) -> DbResult<Vec<MockRow>> {
            Err(ErrorCode::new(500, "db error"))
        }
        fn execute(&mut self, _sql: &str) -> DbResult<u64> {
            Err(ErrorCode::new(500, "db error"))
        }
        fn execute_bind(&mut self, _sql: &str, _id: &dyn ToSqlSync) -> DbResult<u64> {
            Err(ErrorCode::new(500, "db error"))
        }
        fn count(&mut self, _sql: &str) -> DbResult<i64> {
            Err(ErrorCode::new(500, "db error"))
        }
    }

    struct ErrorDao;

    impl SimpleDao<MockRow, ErrorConn> for ErrorDao {
        fn table_name() -> String { "error_table".to_string() }
    }

    #[test]
    fn sync_list_propagates_error() {
        let mut conn = ErrorConn;
        let result = ErrorDao::list(&mut conn);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, 500);
    }

    #[test]
    fn sync_detail_propagates_error() {
        let mut conn = ErrorConn;
        let result = ErrorDao::detail(&1, &mut conn);
        assert!(result.is_err());
    }

    #[test]
    fn sync_delete_propagates_error() {
        let mut conn = ErrorConn;
        let result = ErrorDao::delete(&1, &mut conn);
        assert!(result.is_err());
    }
}