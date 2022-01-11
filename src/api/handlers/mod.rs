pub mod admin;
pub mod user;

use super::errors::internal_error;
use anyhow::Result;
use rweb::*;
use sqlx::PgPool;

#[get("/status")]
#[openapi(tags("System"))]
#[openapi(summary = "Healthcheck")]
#[openapi(description = r#"
Checks db connectivity, returns "OK" on success
"#)]
pub async fn status(#[data] pool: PgPool) -> Result<Json<String>, Rejection> {
    let mut conn = pool.acquire().await.map_err(internal_error)?;
    sqlx::query!(r#"select 'OK' as ok"#)
        .fetch_one(&mut conn)
        .await
        .map(|r| r.ok.unwrap().into())
        .map_err(internal_error)
}
