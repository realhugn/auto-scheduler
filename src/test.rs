#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use crate::health_check;

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/v1")
                        .service(health_check)
                )
        )
            .await;
        let req = test::TestRequest::get().uri("/v1/health_check").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
    }
}