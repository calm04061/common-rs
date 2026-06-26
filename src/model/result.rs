use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct WebResult<T> {
    pub message: String,
    pub code: i32,
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PageResult<T> {
    #[serde(rename(serialize = "currentPage", deserialize = "currentPage"))]
    pub current_page: i64,
    #[serde(rename(serialize = "pageSize", deserialize = "pageSize"))]
    pub page_size: i64,
    #[serde(rename(serialize = "totalCount", deserialize = "totalCount"))]
    pub total_count: i64,
    pub list: Option<Vec<T>>,
}

#[derive(Debug, Deserialize)]
pub struct PageRequest<T> {
    #[serde(rename(deserialize = "currentPage"))]
    pub current_page: i64,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: i64,

    pub query: Option<T>,
}

#[derive(Debug, Clone)]
pub struct ErrorCode {
    pub code: i32,
    pub message: String,
}

pub type DbResult<T> = Result<T, ErrorCode>;

impl ErrorCode {
    pub fn new(code: i32, message: &str) -> Self {
        ErrorCode {
            code,
            message: String::from(message),
        }
    }
    pub fn new_string(code: i32, message: String) -> Self {
        ErrorCode {
            code,
            message,
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = format!("{}:{}", self.code, self.message);
        f.write_str(string.as_str())
    }
}

impl<T: Clone> From<&DbResult<T>> for WebResult<T> {
    fn from(value: &DbResult<T>) -> Self {
        match value {
            Ok(r) => {
                WebResult::success(r.clone())
            }
            Err(e) => {
                WebResult::fail(e.code, e.message.as_str())
            }
        }
    }
}

impl<T: Clone> From<Option<T>> for WebResult<T> {
    fn from(value: Option<T>) -> Self {
        WebResult::new(value)
    }
}

impl<T: Clone> From<DbResult<Option<T>>> for WebResult<T>
{
    fn from(value: DbResult<Option<T>>) -> Self {
        match value {
            Ok(r) => {
                WebResult::new(r)
            }
            Err(e) => {
                WebResult::fail(e.code, e.message.as_str())
            }
        }
    }
}


impl<T: Clone> From<DbResult<T>> for WebResult<T> {
    fn from(value: DbResult<T>) -> Self {
        Self::from(&value)
    }
}

impl<T: Clone> WebResult<T> {
    pub fn new(t: Option<T>) -> WebResult<T> {
        WebResult {
            message: "".to_string(),
            code: 0,
            data: t,
        }
    }
    pub fn success(t: T) -> WebResult<T> {
        WebResult {
            message: "".to_string(),
            code: 0,
            data: Some(t),
        }
    }
    pub fn empty() -> WebResult<T> {
        WebResult {
            message: "".to_string(),
            code: 0,
            data: None,
        }
    }
    pub fn fail(code: i32, message: &str) -> WebResult<T> {
        WebResult {
            message: String::from(message),
            code,
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_result_success() {
        let r: WebResult<i32> = WebResult::success(42);
        assert_eq!(r.code, 0);
        assert!(r.message.is_empty());
        assert_eq!(r.data, Some(42));
    }

    #[test]
    fn web_result_fail() {
        let r: WebResult<i32> = WebResult::fail(1, "not found");
        assert_eq!(r.code, 1);
        assert_eq!(r.message, "not found");
        assert!(r.data.is_none());
    }

    #[test]
    fn web_result_empty() {
        let r: WebResult<i32> = WebResult::empty();
        assert_eq!(r.code, 0);
        assert!(r.data.is_none());
    }

    #[test]
    fn web_result_new_some() {
        let r: WebResult<i32> = WebResult::new(Some(7));
        assert_eq!(r.data, Some(7));
    }

    #[test]
    fn web_result_new_none() {
        let r: WebResult<i32> = WebResult::new(None);
        assert!(r.data.is_none());
    }

    #[test]
    fn from_db_result_ok() {
        let db: DbResult<i32> = Ok(10);
        let web: WebResult<i32> = WebResult::from(&db);
        assert_eq!(web.code, 0);
        assert_eq!(web.data, Some(10));
    }

    #[test]
    fn from_db_result_err() {
        let db: DbResult<i32> = Err(ErrorCode::new(2, "err"));
        let web: WebResult<i32> = WebResult::from(&db);
        assert_eq!(web.code, 2);
        assert_eq!(web.message, "err");
        assert!(web.data.is_none());
    }

    #[test]
    fn from_db_result_owned() {
        let db: DbResult<i32> = Ok(99);
        let web: WebResult<i32> = db.into();
        assert_eq!(web.code, 0);
        assert_eq!(web.data, Some(99));
    }

    #[test]
    fn from_option_some() {
        let web: WebResult<i32> = Some(5).into();
        assert_eq!(web.data, Some(5));
    }

    #[test]
    fn from_option_none() {
        let web: WebResult<i32> = Option::<i32>::None.into();
        assert!(web.data.is_none());
    }

    #[test]
    fn from_db_result_option_ok_some() {
        let db: DbResult<Option<i32>> = Ok(Some(3));
        let web: WebResult<i32> = db.into();
        assert_eq!(web.data, Some(3));
    }

    #[test]
    fn from_db_result_option_ok_none() {
        let db: DbResult<Option<i32>> = Ok(None);
        let web: WebResult<i32> = db.into();
        assert!(web.data.is_none());
    }

    #[test]
    fn from_db_result_option_err() {
        let db: DbResult<Option<i32>> = Err(ErrorCode::new(3, "fail"));
        let web: WebResult<i32> = db.into();
        assert_eq!(web.code, 3);
        assert_eq!(web.message, "fail");
    }

    #[test]
    fn error_code_display() {
        let e = ErrorCode::new(42, "something broke");
        assert_eq!(format!("{}", e), "42:something broke");
    }

    #[test]
    fn error_code_new_string() {
        let e = ErrorCode::new_string(7, "custom".to_string());
        assert_eq!(e.code, 7);
        assert_eq!(e.message, "custom");
    }

    #[test]
    fn page_request_deserialize_current_page() {
        let json = r#"{"currentPage":3,"pageSize":20}"#;
        let req: PageRequest<()> = serde_json::from_str(json).unwrap();
        assert_eq!(req.current_page, 3);
        assert_eq!(req.page_size, 20);
        assert!(req.query.is_none());
    }

    #[test]
    fn page_result_serialize_camel_case() {
        let pr = PageResult {
            current_page: 1,
            page_size: 10,
            total_count: 100,
            list: Some(vec!["a".to_string()]),
        };
        let json = serde_json::to_string(&pr).unwrap();
        assert!(json.contains(r#""currentPage""#));
        assert!(json.contains(r#""pageSize""#));
        assert!(json.contains(r#""totalCount""#));
    }
}
