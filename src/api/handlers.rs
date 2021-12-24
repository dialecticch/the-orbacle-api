use super::errors::internal_error;
use crate::profiles::token_profile::TokenProfile;
use anyhow::Result;
use rweb::*;
use sqlx::PgPool;

#[get("/status")]
#[openapi(tags("system"))]
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

#[get("/profile/{collection}/{token_id}")]
#[openapi(tags("system"))]
#[openapi(summary = "Get Token Pricing Profile")]
#[openapi(description = r#"
Fetches token data and returns a token profile
"#)]
pub async fn get_profile(
    #[data] pool: PgPool,
    token_id: i32,
    collection: String,
) -> Result<Json<TokenProfile>, Rejection> {
    println!("/get_profile/{}/{}", collection, token_id);
    let mut conn = pool.acquire().await.map_err(internal_error)?;

    TokenProfile::make(&mut conn, &collection.to_string(), token_id, 1)
        .await
        .map(|r| r.into())
        .map_err(internal_error)
}
