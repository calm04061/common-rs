use crate::model::result::{DbResult, ErrorCode, PageRequest, PageResult};
use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{query, Executor, Pool, Postgres, Row, Transaction};
use sqlx::AssertSqlSafe;

pub type DbPool = Pool<Postgres>;

fn to_ec(e: sqlx::Error) -> ErrorCode {
    ErrorCode::new(1, &e.to_string())
}

#[async_trait]
pub trait SimpleDao<E>
where
    E: From<PgRow> + Sync,
{
    fn table_name() -> String;

    async fn page(
        page_request: &PageRequest<E>,
        conn: &mut Transaction<'_, Postgres>,
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
            "SELECT * FROM {} order by id limit $1 offset $2",
            Self::table_name()
        );
        let result = conn
            .fetch_all(sqlx::query(AssertSqlSafe(sql)).bind(page_size_).bind(offset))
            .await
            .map_err(to_ec)?;
        let result = Self::convert(Ok(result));

        Ok(PageResult {
            current_page: page_request.current_page,
            page_size: page_request.page_size,
            total_count: count,
            list: Some(result?),
        })
    }

    async fn list(tran: &mut Transaction<'_, Postgres>) -> DbResult<Vec<E>> {
        let sql = format!("select * from {} ", Self::table_name());
        let query: query::Query<'_, Postgres, _> =
            sqlx::query(AssertSqlSafe(sql));
        let vec = tran
            .fetch_all::<query::Query<'_, Postgres, _>>(query)
            .await
            .map_err(to_ec)?;
        Self::convert(Ok(vec))
    }

    async fn detail<'q>(id: i32, tran: &mut Transaction<'_, Postgres>) -> DbResult<Option<E>> {
        let sql = format!("select * from {} where id =$1", Self::table_name());
        let query = sqlx::query(AssertSqlSafe(sql)).bind(id);
        let result = tran.fetch_all(query).await.map_err(to_ec)?;
        Self::get_first(Ok(result))
    }
    async fn delete(id: i32, conn: &mut Transaction<'_, Postgres>) -> DbResult<u64> {
        let sql = format!("delete from {} where id = $1", Self::table_name());
        let query = sqlx::query(AssertSqlSafe(sql)).bind(id);
        Ok(conn.execute(query).await.map_err(to_ec)?.rows_affected())
    }
    fn convert(result: DbResult<Vec<PgRow>>) -> DbResult<Vec<E>> {
        convert(result)
    }
    fn get_first(result: DbResult<Vec<PgRow>>) -> DbResult<Option<E>> {
        get_first(result)
    }
}

pub fn get_first<T: From<PgRow>>(result: DbResult<Vec<PgRow>>) -> DbResult<Option<T>> {
    if let Err(e) = result {
        return Err(e);
    }
    let vec = result?
        .into_iter()
        .map(|r| T::from(r))
        .collect::<Vec<T>>();
    if vec.is_empty() {
        Ok(None)
    } else {
        let data = vec.into_iter().next();
        Ok(data)
    }
}

pub fn convert<T: From<PgRow>>(result: DbResult<Vec<PgRow>>) -> DbResult<Vec<T>> {
    if let Err(e) = result {
        return Err(e);
    }
    let vec = result?
        .into_iter()
        .map(|r| T::from(r))
        .collect::<Vec<T>>();
    Ok(vec)
}
