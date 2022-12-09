use actix_web::web::{self, Form};
use actix_web::HttpResponse;
use chrono::Utc;
use sqlx::types::Uuid;
use sqlx::PgPool;
use tracing::log::{self, log};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name=%form.name
    )
)]
pub async fn subscribe(form: Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    log::info!("Saving new subscriber details to the database");
    match insert_subscriber(&pool, &form).await {
        Ok(_) => {
            log::info!("New subsriber details have been saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!("{:?}: Failed to execute query", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT into subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;
    Ok(())
}
