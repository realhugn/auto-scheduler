use actix_web::{Error, HttpResponse, web};
use serde_json::json;
use crate::config::postgres::DbPool;
use crate::models::employee::{Employee, EmployeeDTO, LoginDTO};
use crate::response::match_err_response;

pub async fn create(pool: web::Data<DbPool>, payload: web::Json<EmployeeDTO>) -> Result<HttpResponse, Error> {
    let result = web::block(move || {
        let mut conn = pool.get()?;
        //add service module later
        Employee::new(payload.into_inner(), &mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);
    match_err_response(result)
}

pub async fn seed(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let pool1 = pool.clone();
    let payload = EmployeeDTO {
        name: "Manager".to_string(),
        email: "manager@vsec.com.vn".to_string(),
        password: "123".to_string(),
        phone_number: None,
        department: None,
        role: "Manager".to_string(),
        availability: None,
    };
    let _result = web::block(move || {
        let mut conn = pool1.clone().get()?;
        //add service module later
        Employee::new(payload, &mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);
    for i in 0..20 {
        let payload = EmployeeDTO {
            name: format!("SOC{}", i),
            email: format!("SOC{}@vsec.com.vn", i),
            password: "123".to_string(),
            phone_number: None,
            department: None,
            role: "SOC".to_string(),
            availability: None,
        };
        let pool_shared = pool.clone();
        let _result = web::block(move || {
            let mut conn = pool_shared.get()?;
            //add service module later
            Employee::new(payload, &mut conn)
        }).await?.map_err(actix_web::error::ErrorInternalServerError);
    }

    match_err_response(Ok("ok"))
}

pub async fn login(login_dto: web::Json<LoginDTO>, pool: web::Data<DbPool>) ->   Result<HttpResponse, Error> {
    let rs = web::block(move || {
        let mut conn = pool.get()?;
        Employee::login(login_dto.into_inner(), &mut conn)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError);

    // response OK if user added , FAIL if some server, database or validation error occur
    match rs {
        Ok(token) => Ok(HttpResponse::Ok().json(json!({"access_token" : token}))),
        Err(_) => {
            return Ok(HttpResponse::BadRequest().body("FAIL"))
        }
    }
}

pub async fn get_by_id(uid: web::Path<i32> ,pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let uid = uid.into_inner();
    let rs = web::block(move || {
        let mut conn = pool.get()?;
        Employee::find_by_id(uid, &mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);

    match_err_response(rs)
}

pub async fn get_managers(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let rs = web::block(move || {
        let mut conn = pool.get()?;
        Employee::find_by_role("Manager", &mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);

    match_err_response(rs)
}

pub async fn get_employees(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let rs = web::block(move || {
        let mut conn = pool.get()?;
        Employee::find_by_role("SOC", &mut conn)
    }).await?.map_err(actix_web::error::ErrorInternalServerError);

    match_err_response(rs)
}

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/user")
        .route("/", web::post().to(create))
        .route("/login", web::post().to(login))
        .route("/seed", web::get().to(seed))
        .route("/employees", web::get().to(get_employees))
        .route("/managers", web::get().to(get_managers))
        .route("/{id}" , web::get().to(get_by_id));
    conf.service(scope);
}

