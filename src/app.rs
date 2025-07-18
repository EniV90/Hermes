use std::net::TcpListener;

use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use actix_web::middleware::Logger;

pub fn run(_listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    println!("Starting server on http://127.0.0.1:8080");
    let db_pool: web::Data<PgPool> = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
        .wrap(Logger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .listen(_listener)?
    .run();
    Ok(server)
}
