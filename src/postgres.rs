#[cfg(feature = "postgres")]
pub mod connection;
#[cfg(feature = "postgres")]
pub mod dao;
#[cfg(feature = "postgres")]
pub mod controller;

use log::error;
use crate::model::result::WebResult;
use crate::postgres::connection::DbResult;


pub fn to_web_result<T>(result: DbResult<T>) -> WebResult<T> {
    return match result {
        Ok(obj) => {
            WebResult {
                message: "".to_string(),
                code: 0,
                data: Some(obj),
            }
        }
        Err(e) => {
            error!("{:?}", e);
            WebResult {
                message: e.to_string(),
                code: 1,
                data: None,
            }
        }
    };
}

pub fn to_web_option<T>(result: Option<T>) -> WebResult<T> {
    WebResult {
        message: "".to_string(),
        code: 0,
        data: result,
    }
}
