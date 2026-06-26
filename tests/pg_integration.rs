#![cfg(feature = "sqlx_pg")]

use common::dao::async_dao::PgAsyncTran;
use common::dao::sqlx_postgres::DbPool;
use sqlx::{AssertSqlSafe, Executor, Row};

fn load_env() {
    dotenv::dotenv().ok();
    let cwd = std::env::current_dir().unwrap_or_default();
    let workspace_env = cwd.join("../.env");
    if workspace_env.exists() {
        dotenv::from_path(&workspace_env).ok();
    }
}

async fn get_pool() -> DbPool {
    load_env();
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. postgres://postgres:root123@127.0.0.1:5432/manager)");
    sqlx::PgPool::connect(&url).await.unwrap()
}

/// Call `Executor::execute` on the inner `Transaction` (avoids `AsyncConnection` ambiguity).
async fn exec_sql(tran: &mut PgAsyncTran<'_>, stmt: &str) {
    let sql = stmt.to_string();
    tran.0.execute(sqlx::query(AssertSqlSafe(sql))).await.unwrap();
}

/// These integration tests require a running PostgreSQL and DATABASE_URL.
/// Run: DATABASE_URL=postgres://postgres:root123@127.0.0.1:5432/manager cargo test --features sqlx_pg,web -- --ignored

#[ignore]
#[tokio::test]
async fn begin_then_rollback() {
    let pool = get_pool().await;
    let mut tran = PgAsyncTran::begin(&pool).await.unwrap();
    let row = tran.fetch_one(sqlx::query("SELECT 1 AS v")).await.unwrap();
    assert_eq!(row.get::<i32, _>("v"), 1);
    tran.rollback().await.unwrap();
}

#[ignore]
#[tokio::test]
async fn commit_persists_data() {
    let pool = get_pool().await;
    let mut tran = PgAsyncTran::begin(&pool).await.unwrap();
    exec_sql(&mut tran, "CREATE TEMP TABLE test_commit (id INT PRIMARY KEY, val TEXT)").await;
    exec_sql(&mut tran, "INSERT INTO test_commit VALUES (1, 'hello')").await;
    tran.commit().await.unwrap();

    let mut tran = PgAsyncTran::begin(&pool).await.unwrap();
    let row = tran.fetch_one(sqlx::query("SELECT val FROM test_commit WHERE id = 1")).await.unwrap();
    assert_eq!(row.get::<String, _>("val"), "hello");
    exec_sql(&mut tran, "DROP TABLE test_commit").await;
    tran.commit().await.unwrap();
}

#[ignore]
#[tokio::test]
async fn rollback_discards_data() {
    let pool = get_pool().await;
    let mut tran = PgAsyncTran::begin(&pool).await.unwrap();
    exec_sql(&mut tran, "CREATE TEMP TABLE test_rollback (id INT)").await;
    exec_sql(&mut tran, "INSERT INTO test_rollback VALUES (99)").await;
    tran.rollback().await.unwrap();

    let mut tran = PgAsyncTran::begin(&pool).await.unwrap();
    let row = tran.fetch_one(sqlx::query("SELECT COUNT(*) AS c FROM test_rollback")).await.unwrap();
    assert_eq!(row.get::<i64, _>("c"), 0);
    exec_sql(&mut tran, "DROP TABLE test_rollback").await;
    tran.commit().await.unwrap();
}

#[cfg(feature = "web")]
mod after_transaction_tests {
    use super::*;
    use common::web::sqlx_postgres::after_transaction;
    use common::model::result::{DbResult, ErrorCode};

    #[ignore]
    #[tokio::test]
    async fn commits_on_ok() {
        let pool = get_pool().await;
        let tran = PgAsyncTran::begin(&pool).await.unwrap();
        let result: DbResult<i32> = Ok(42);
        let result = after_transaction(result, tran).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[ignore]
    #[tokio::test]
    async fn rolls_back_on_err() {
        let pool = get_pool().await;
        let tran = PgAsyncTran::begin(&pool).await.unwrap();
        let result: DbResult<i32> = Err(ErrorCode::new(500, "test error"));
        let result = after_transaction(result, tran).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, 500);
    }
}

mod async_connection_tests {
    use super::*;
    use common::dao::async_dao::AsyncConnection;

    struct TestRow(i32, String);

    impl From<sqlx::postgres::PgRow> for TestRow {
        fn from(row: sqlx::postgres::PgRow) -> Self {
            TestRow(row.get("id"), row.get("val"))
        }
    }

    #[ignore]
    #[tokio::test]
    async fn query_all_returns_rows() {
        let pool = get_pool().await;
        let mut tran = PgAsyncTran::begin(&pool).await.unwrap();
        exec_sql(&mut tran, "CREATE TEMP TABLE test_qa (id INT, val TEXT)").await;
        exec_sql(&mut tran, "INSERT INTO test_qa VALUES (1, 'a'), (2, 'b')").await;

        let rows = AsyncConnection::<TestRow>::query_all(&mut tran, "SELECT id, val FROM test_qa ORDER BY id").await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, 1);
        assert_eq!(rows[1].1, "b");

        tran.rollback().await.unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn execute_returns_affected_rows() {
        let pool = get_pool().await;
        let mut tran = PgAsyncTran::begin(&pool).await.unwrap();
        exec_sql(&mut tran, "CREATE TEMP TABLE test_exec (id INT)").await;
        exec_sql(&mut tran, "INSERT INTO test_exec VALUES (1), (2), (3)").await;

        let count = AsyncConnection::<TestRow>::execute(&mut tran, "DELETE FROM test_exec").await.unwrap();
        assert_eq!(count, 3);

        tran.rollback().await.unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn count_returns_row_count() {
        let pool = get_pool().await;
        let mut tran = PgAsyncTran::begin(&pool).await.unwrap();
        exec_sql(&mut tran, "CREATE TEMP TABLE test_cnt (id INT)").await;
        exec_sql(&mut tran, "INSERT INTO test_cnt VALUES (10), (20)").await;

        let n = AsyncConnection::<TestRow>::count(&mut tran, "SELECT count(1) FROM test_cnt").await.unwrap();
        assert_eq!(n, 2);

        tran.rollback().await.unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn query_bind_filters_by_id() {
        let pool = get_pool().await;
        let mut tran = PgAsyncTran::begin(&pool).await.unwrap();
        exec_sql(&mut tran, "CREATE TEMP TABLE test_qb (id INT, val TEXT)").await;
        exec_sql(&mut tran, "INSERT INTO test_qb VALUES (1, 'x'), (2, 'y')").await;

        let rows = AsyncConnection::<TestRow>::query_bind(&mut tran, "SELECT id, val FROM test_qb WHERE id = $1", &1i32).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, 1);

        tran.rollback().await.unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn execute_bind_deletes_by_id() {
        let pool = get_pool().await;
        let mut tran = PgAsyncTran::begin(&pool).await.unwrap();
        exec_sql(&mut tran, "CREATE TEMP TABLE test_eb (id INT)").await;
        exec_sql(&mut tran, "INSERT INTO test_eb VALUES (1), (2)").await;

        let n = AsyncConnection::<TestRow>::execute_bind(&mut tran, "DELETE FROM test_eb WHERE id = $1", &1i32).await.unwrap();
        assert_eq!(n, 1);

        tran.rollback().await.unwrap();
    }
}
