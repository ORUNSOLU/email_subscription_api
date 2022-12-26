use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use std::net::TcpListener;
use crate::routes::{health_check, subscribe};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;


pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // wrap the pool using WEB::DaTA which boils down to an ARC smart pointer
    let db_pool = web::Data::new(db_pool);
    // capture `connection` from the surrounding environment
    let server = HttpServer::new(move || {
        App::new()
            // middlewares are added using the WRAP method
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // Get a pointer copy and attach it to the connection state
            .app_data(db_pool.clone())
    }).listen(listener)?.run();
    Ok(server)
}

// convert from JSON object to....
// #[derive(serde::Deserialize)]
// struct FormData {
//     email: String,
//     name: String
// }

// async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
//     HttpResponse::Ok().finish()
// }

// async fn health_check() -> HttpResponse {
//     HttpResponse::Ok().finish()
// }
