use actix_web::{Error, HttpResponse, web};
use crate::config::postgres::DbPool;
use crate::middleware;
use crate::models::shift_changes::{ShiftChange, ShiftChangeDTO};
use crate::response::match_err_response;

pub async fn create(pool: web::Data<DbPool>, payload: web::Json<ShiftChangeDTO>) -> Result<HttpResponse, Error> {
    let result = web::block(move || {
        let mut conn = pool.get()?;
        //add service module later
        ShiftChange::new(payload.into_inner(), &mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);
    match_err_response(result)
}

pub async fn verify(pool: web::Data<DbPool>, shift_change_id: web::Path<i32>) ->  Result<HttpResponse, Error> {
    let result = web::block(move || {
        let mut conn = pool.get()?;
        //add service module later
        ShiftChange::verify_change(shift_change_id.into_inner(), &mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);
    match_err_response(result)
}

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/shift_change").wrap(middleware::jwt::JWTAuth)
        .route("/", web::post().to(create))
        .route("/verify/{id}", web::get().to(verify).wrap(middleware::is_manager::IsManager));
    conf.service(scope);
}
