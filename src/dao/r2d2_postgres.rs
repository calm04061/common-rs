#[cfg(feature = "r2d2_pg")]
pub mod connection;
#[cfg(feature = "r2d2_pg")]
pub mod dao;

use log::error;
use crate::model::result::{DbResult, WebResult};

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
