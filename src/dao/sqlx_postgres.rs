use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{Executor, Pool, Postgres, query, Row, Transaction};
use crate::model::result::{DbResult, PageRequest, PageResult};

pub type DbPool = Pool<Postgres>;

#[async_trait]
pub trait SimpleDao<E>
    where E: From<PgRow> + Sync
{
    fn table_name() -> String;

    async fn page(page_request: &PageRequest<E>, conn: &mut Transaction<'_, Postgres>) -> DbResult<PageResult<E>> {
        let sql = format!("SELECT count(1) FROM {} ", Self::table_name());
        let sql = sql.as_str();
        let query = sqlx::query(sql);
        let row = conn.fetch_one::<query::Query<'_, Postgres, _>>(query).await.unwrap();
        let count: i64 = row.get(0);
        // let count: i64 = conn.query_one(format!("SELECT count(1) FROM {} ", Self::table_name()).as_str(), &[])?.get(0);
        let page_size_ = page_request.page_size;
        let offset = (page_request.current_page - 1) * page_request.page_size;
        let sql = format!("SELECT * FROM {} order by id limit $1 offset $2", Self::table_name());
        let sql = sql.as_str();
        let query = sqlx::query(sql)
            .bind(page_size_)
            .bind(offset);
        let result = conn.fetch_all(query).await.unwrap();
        let result = Self::convert(Ok(result));

        Ok(PageResult {
            current_page: page_request.current_page,
            page_size: page_request.page_size,
            total_count: count,
            list: Some(result.unwrap()),
        })
    }

    async fn list(tran: &mut Transaction<'_, Postgres>) -> DbResult<Vec<E>> {
        let sql = format!("select * from {} ", Self::table_name());
        let sql = sql.as_str();
        let query: query::Query<'_, Postgres, _> = query(sql);
        let vec = tran.fetch_all::<query::Query<'_, Postgres, _>>(query).await.unwrap();
        Self::convert(Ok(vec))
    }

    async fn detail<'q>(id: i32, tran: &mut Transaction<'_, Postgres>) -> DbResult<Option<E>> {
        let sql = format!("select * from {} where id =$1", Self::table_name());
        let sql = sql.as_str();
        let query = query(sql)
            .bind(id);
        let result = tran.fetch_all(query).await.unwrap();
        Self::get_first(Ok(result))
    }
    async fn delete(id: i32, conn: &mut Transaction<'_, Postgres>) -> DbResult<u64> {
        let sql = format!("delete from {} where id = $1", Self::table_name());
        let sql = sql.as_str();
        let query = query(sql)
            .bind(id);
        Ok(conn.execute(query).await.unwrap().rows_affected())
    }
    fn convert(result: DbResult<Vec<PgRow>>) -> DbResult<Vec<E>> {
        convert(result)
    }
    fn get_first(result: DbResult<Vec<PgRow>>) -> DbResult<Option<E>> {
        get_first(result)
    }
}

pub fn get_first<T: From<PgRow>>(result: DbResult<Vec<PgRow>>) -> DbResult<Option<T>>
{
    if let Err(e) = result {
        return Err(e);
    }
    let vec = result.unwrap()
        .into_iter().map(|r| {
        T::from(r)
    }).collect::<Vec<T>>();
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
    let vec = result.unwrap()
        .into_iter().map(|r| {
        T::from(r)
    }).collect::<Vec<T>>();
    Ok(vec)
}
