use std::net::TcpListener;

use crate::routes::{health_check, subscribe};
use actix_web::{App, HttpServer, dev::Server, web};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub fn run(_listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool: web::Data<PgPool> = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .listen(_listener)?
    .run();
    Ok(server)
}
