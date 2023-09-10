use r2d2_postgres::postgres::{Row, Transaction};
use crate::model::result::{PageRequest, PageResult};
use crate::postgres::connection::DbResult;

pub trait SimpleDao<T: From<Row>, I: Sync + tokio_postgres::types::ToSql> {
    fn table_name() -> String;

    fn page(page_request: &PageRequest<T>, conn: &mut Transaction) -> DbResult<PageResult<T>> {
        let count: i64 = conn.query_one(format!("SELECT count(1) FROM {} ", Self::table_name()).as_str(), &[])?.get(0);
        let page_size_ = page_request.page_size as i64;
        let offset = ((page_request.current_page - 1) * page_request.page_size) as i64;
        let result = conn.query(format!("SELECT * FROM {} order by id limit $1 offset $2", Self::table_name()).as_str(), &[&page_size_, &offset]);
        let result = Self::convert(result);

        Ok(PageResult {
            current_page: page_request.current_page,
            page_size: page_request.page_size,
            total_count: count,
            list: Some(result.unwrap()),
        })
    }

    fn list(tran: &mut Transaction) -> DbResult<Vec<T>> {
        let result = tran.query(format!("select * from {} ", Self::table_name()).as_str(), &[]);
        Self::convert(result)
    }

    fn detail(id: I, tran: &mut Transaction) -> DbResult<Option<T>> {
        let result = tran.query(format!("select * from {} where id =$1", Self::table_name()).as_str(), &[&id]);
        Self::get_first(result)
    }

    fn delete(id: I, conn: &mut Transaction) -> DbResult<u64> {
        Ok(conn.execute(format!("delete from {} where id = $1", Self::table_name()).as_str(), &[&id]).unwrap())
    }
    fn convert(result: DbResult<Vec<Row>>) -> DbResult<Vec<T>> {
        convert(result)
    }
    fn get_first(result: DbResult<Vec<Row>>) -> DbResult<Option<T>> {
        get_first(result)
    }
}

pub fn get_first<T: From<Row>>(result: DbResult<Vec<Row>>) -> DbResult<Option<T>>
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

pub fn convert<T: From<Row>>(result: DbResult<Vec<Row>>) -> DbResult<Vec<T>> {
    if let Err(e) = result {
        return Err(e);
    }
    let vec = result.unwrap()
        .into_iter().map(|r| {
        T::from(r)
    }).collect::<Vec<T>>();
    Ok(vec)
}
