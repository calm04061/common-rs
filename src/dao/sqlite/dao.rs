use r2d2_sqlite::rusqlite::{Row, Rows, Transaction};
use crate::dao::sqlite::connection::DbResult;

pub trait SimpleDao<T: for<'a> From<&'a Row<'a>>, I: r2d2_sqlite::rusqlite::ToSql> {
    fn table_name() -> String;
    fn list(tran: &mut Transaction) -> DbResult<Vec<T>> {
        let mut statement = tran.prepare(format!("select * from {} ", Self::table_name()).as_str()).unwrap();
        let result = statement.query([]);

        Self::convert(result)
    }

    fn detail(id: I, tran: &mut Transaction) -> DbResult<Option<T>> {
        let mut statement = tran.prepare(format!("select * from {} where id =?", Self::table_name()).as_str()).unwrap();

        let result = statement.query([id]);
        Self::get_first(result)
    }

    fn delete(id: I, conn: &mut Transaction) -> DbResult<usize> {
        let mut statement = conn.prepare(format!("update {} set deleted = 1 where id = ?", Self::table_name()).as_str()).unwrap();
        statement.execute([id])
    }
    fn convert(result: DbResult<Rows>) -> DbResult<Vec<T>> {
        if let Err(e) = result {
            return Err(e);
        }
        let mut rows = result.unwrap();
        let mut vec = vec![];
        while let Some(row) = rows.next().unwrap() {
            vec.push(T::from(row))
        }
        Ok(vec)
    }
    fn get_first(result: DbResult<Rows>) -> DbResult<Option<T>> {
        let vec = Self::convert(result).unwrap();
        if vec.is_empty() {
            Ok(None)
        } else {
            let data = vec.into_iter().next();
            Ok(data)
        }
    }
}
