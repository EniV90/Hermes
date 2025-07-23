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

pub async fn subscribe(_form: web::Form<Info>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_name = %_form.name,
        subscriber_email = %_form.email
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details into the database.");
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
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber detail has been saved into the database",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("request_id {} - Failed to execute query {e:?}", request_id);
            HttpResponse::InternalServerError().finish()
        }
    }
}
