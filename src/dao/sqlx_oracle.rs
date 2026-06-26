use crate::model::result::{DbResult, ErrorCode, PageRequest, PageResult};
use async_trait::async_trait;
use sqlx::{Executor, Pool, Row, Transaction};
use sqlx::AssertSqlSafe;
use sqlx_oracle::Oracle;

pub type DbPool = Pool<Oracle>;

fn to_ec(e: sqlx::Error) -> ErrorCode {
    ErrorCode::new(1, &e.to_string())
}

#[async_trait]
pub trait SimpleDao<E>
where
    E: for<'r> From<&'r <Oracle as sqlx::Database>::Row> + Sync,
{
    fn table_name() -> String;

    async fn page(
        page_request: &PageRequest<E>,
        conn: &mut Transaction<'_, Oracle>,
    ) -> DbResult<PageResult<E>> {
        let sql = format!("SELECT count(1) FROM {} ", Self::table_name());
        let row = conn
            .fetch_one(sqlx::query(AssertSqlSafe(sql)))
            .await
            .map_err(to_ec)?;
        let count: i64 = row.get(0);
        let page_size_ = page_request.page_size;
        let offset = (page_request.current_page - 1) * page_request.page_size;
        let sql = format!(
            "SELECT * FROM {} order by id offset {} rows fetch next {} rows only",
            Self::table_name(),
            offset,
            page_size_
        );
        let rows = conn
            .fetch_all(sqlx::query(AssertSqlSafe(sql)))
            .await
            .map_err(to_ec)?;
        let list = rows.iter().map(|r| E::from(r)).collect::<Vec<E>>();

        Ok(PageResult {
            current_page: page_request.current_page,
            page_size: page_request.page_size,
            total_count: count,
            list: Some(list),
        })
    }

    async fn list(tran: &mut Transaction<'_, Oracle>) -> DbResult<Vec<E>> {
        let sql = format!("select * from {} ", Self::table_name());
        let rows = tran
            .fetch_all(sqlx::query(AssertSqlSafe(sql)))
            .await
            .map_err(to_ec)?;
        Ok(rows.iter().map(|r| E::from(r)).collect())
    }

    async fn detail(id: i32, tran: &mut Transaction<'_, Oracle>) -> DbResult<Option<E>> {
        let sql = format!("select * from {} where id =$1", Self::table_name());
        let rows = tran
            .fetch_all(sqlx::query(AssertSqlSafe(sql)).bind(id))
            .await
            .map_err(to_ec)?;
        Ok(rows.first().map(|r| E::from(r)))
    }

    async fn delete(id: i32, conn: &mut Transaction<'_, Oracle>) -> DbResult<u64> {
        let sql = format!("delete from {} where id = $1", Self::table_name());
        Ok(conn
            .execute(sqlx::query(AssertSqlSafe(sql)).bind(id))
            .await
            .map_err(to_ec)?
            .rows_affected())
    }
}
