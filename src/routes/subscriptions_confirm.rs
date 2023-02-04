use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
pub async fn confirm(pool: web::Data<PgPool>, parameters: web::Query<Parameters>) -> HttpResponse {
    let subscription_id = match get_subscription_id(&parameters.subscription_token, &pool).await {
        Ok(subscription_id) => subscription_id,
        Err(e) => return HttpResponse::InternalServerError().finish(),
    };

    match subscription_id {
        None => HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => {
            if confirm_subscriber(&pool, subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
    }
}

async fn get_subscription_id(
    subscription_token: &str,
    pool: &PgPool,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"select subscriber_id from subscription_tokens where subscription_token= $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {}", e);
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}

async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"update subscriptions set status = 'confirmed' where id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
