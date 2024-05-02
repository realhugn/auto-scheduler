use actix_web::{Error, HttpResponse};
use serde::{Deserialize, Serialize};

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseBody<T> {
    pub message: String,
    pub data: T,
}

impl<T> ResponseBody<T> {
    pub fn new(message: &str, data: T) -> ResponseBody<T> {
        ResponseBody {
            message: message.to_string(),
            data,
        }
    }
}

#[allow(unused)]
#[derive(Serialize)]
pub struct Page<T> {
    pub message: String,
    pub data: Vec<T>,
    pub page_num: i64,
    pub page_size: i64,
    pub total_elements: i64,
}

#[allow(unused)]
impl<T> Page<T> {
    pub fn new(
        message: &str,
        data: Vec<T>,
        page_num: i64,
        page_size: i64,
        total_elements: i64,
    ) -> Page<T> {
        Page {
            message: message.to_string(),
            data,
            page_num,
            page_size,
            total_elements,
        }
    }
}

pub fn match_err_response<T: serde::Serialize>(result: Result<T, Error>) ->  Result<HttpResponse, Error>{
    match result {
        Ok(rs) => Ok(HttpResponse::Ok().json(rs)),
        Err(err) => {
            return Ok(HttpResponse::BadRequest().body(err.to_string()))
        }
    }
}

