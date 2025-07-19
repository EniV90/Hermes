#![allow(unused_imports)]
use actix_web::{HttpResponse, web};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Info {
    email: String,
    name: String,
}

pub async fn subscribe(_form: web::Form<Info>, pool: web::Data<PgPool>) -> HttpResponse {
    log::info!("Adding '{}', '{}' as a new member", _form.name, _form.email);
    log::info!("Saving new subscriber details into the database.");
    match sqlx::query!(
        r#"
      INSERT INTO subscriptions (id, email, name, subscribed_at)
      VALUES ($1, $2, $3, $4)
      "#,
        Uuid::new_v4(),
        _form.email,
        _form.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            log::info!("New subscriber detail has been saved into the database");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!("Failed to execute query {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
