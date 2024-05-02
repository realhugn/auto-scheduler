use actix_cors::Cors;
#[allow(unused)]
use actix_web::{App, get, HttpServer, web};
use actix_web::middleware::Logger;
use crate::config::postgres::establish_connection_pool;


mod config;
mod test;
mod schema;
mod models;
mod constants;
mod error;
mod response;
mod route;
mod utils;
mod middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    // Create postgres connection pool
    let pool = establish_connection_pool();
    HttpServer::new(
        move || {
            let cors = Cors::permissive();

            App::new()
                .wrap(cors)
                .app_data(web::Data::new(pool.clone()))
                .wrap(Logger::default())
                .service(
                    web::scope("/v1")
                        .configure(route::employee::config)
                        .configure(route::schedule::config)
                        .configure(route::shift_change::config)
                        .service(health_check)
                )
        }
    )
    .bind(("0.0.0.0", 5001))?
    .run()
    .await
}

#[get("/health_check")]
async fn health_check() -> String {
     "OK".to_string()
}

