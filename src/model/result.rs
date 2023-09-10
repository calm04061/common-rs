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

unsafe impl Send for ErrorCode {}

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
