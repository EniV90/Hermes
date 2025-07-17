use std::net::TcpListener;

use crate::routes::{health_check, subscribe};
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;

pub async fn run(_listener: TcpListener, db_pool: PgPool) -> std::io::Result<()> {
    println!("Starting server on http://127.0.0.1:8080");
    let db_pool: web::Data<PgPool> = web::Data::new(db_pool);
    HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
