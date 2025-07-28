#![allow(unused_imports)]
use actix_web::{HttpResponse, web};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Info {
    email: String,
    name: String,
}

#[tracing::instrument(
    name=  "Adding a new subscriber",
    skip(_form, pool),
    fields (
        subscriber_name = %_form.name,
        subscriber_email = %_form.email
    )
)]

pub async fn subscribe(_form: web::Form<Info>, pool: web::Data<PgPool>) -> HttpResponse {
    match insert_subscriber(&pool, &_form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Saving new subscriber details in database", skip(_form, pool))]

pub async fn insert_subscriber(pool: &PgPool, _form: &Info) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
      INSERT INTO subscriptions (id, email, name, subscribed_at)
      VALUES ($1, $2, $3, $4)
      "#,
        Uuid::new_v4(),
        _form.email,
        _form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {e:?}");
        e
    })?;
    Ok(())
}
