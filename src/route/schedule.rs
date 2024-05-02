use actix_web::{Error, HttpResponse, web};
use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use serde::Deserialize;
use web::Json;
use crate::config::postgres::DbPool;
use crate::middleware;
use crate::models::schedule::{AutoScheduleDTO, Schedule, ScheduleDTO};
use crate::response::match_err_response;
use mime::Mime;

pub async fn create(pool: web::Data<DbPool>, payload: Json<ScheduleDTO>) -> Result<HttpResponse, Error> {
    let result = web::block(move || {
        let mut conn = pool.get()?;
        //add service module later
        Schedule::new(payload.into_inner(), &mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);
    match_err_response(result)
}

pub async fn generate_schedules(pool: web::Data<DbPool>, payload: Json<AutoScheduleDTO>) -> Result<HttpResponse, Error>{
    let rs = web::block(move || {
        let mut conn = pool.get()?;
        Schedule::from_sample_to_db(payload.into_inner(),&mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);
    match_err_response(rs)
}

#[derive(Deserialize)]
pub struct Info {
    pub month: i32,
    pub year: i32
}

pub async fn export_csv(param : web::Query<Info>, pool: web::Data<DbPool>) -> Result<NamedFile, Error> {
    let rs = web::block(move || {
        let mut conn = pool.get()?;
        Schedule::export_csv(param.month, param.year, &mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);

    match rs {
        Ok(filename) => {
            let file = NamedFile::open(&filename)?;

            let content_disposition = ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![
                    DispositionParam::Filename(String::from(filename))
                ],
            };

            let content_type: Mime = "text/csv".parse().unwrap();
            Ok(file
                .set_content_disposition(content_disposition)
                .set_content_type(content_type))
        },
        Err(err) => {
            return Err(err)
        }
    }
}

pub async fn get_by_month(param : web::Query<Info>, pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let rs = web::block(move || {
        let mut conn = pool.get()?;
        Schedule::get_by_month_year(param.month, param.year, &mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);

    match_err_response(rs)
}

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/schedule")
        .route("/", web::get().to(get_by_month).wrap(middleware::jwt::JWTAuth))
        .route("/", web::post().to(create).wrap(middleware::is_manager::IsManager).wrap(middleware::jwt::JWTAuth))
        .route("/gen", web::post().to(generate_schedules).wrap(middleware::is_manager::IsManager).wrap(middleware::jwt::JWTAuth))
        .route("/export", web::get().to(export_csv))
      ;
    conf.service(scope);
}