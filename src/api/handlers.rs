use super::errors::internal_error;
use crate::analyzers::rarities::get_trait_rarities;
use crate::profiles::token::price_profile::PriceProfile;
use crate::profiles::token::token_profile::TokenProfile;
use crate::storage::read::read_collection;
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

#[get("/profile/{collection_slug}/{token_id}")]
#[openapi(tags("system"))]
#[openapi(summary = "Get Token Profile")]
#[openapi(description = r#"
Fetches token data and returns a token profile
"#)]
pub async fn get_profile(
    #[data] pool: PgPool,
    token_id: i32,
    collection_slug: String,
) -> Result<Json<TokenProfile>, Rejection> {
    println!("/get_profile/{}/{}", collection_slug, token_id);
    let mut conn = pool.acquire().await.map_err(internal_error)?;

    let collection = read_collection(&mut conn, &collection_slug)
        .await
        .map_err(internal_error)?;

    TokenProfile::make(&mut conn, collection, token_id)
        .await
        .map(|r| r.into())
        .map_err(internal_error)
}

#[get("/price/{collection_slug}/{token_id}")]
#[openapi(tags("system"))]
#[openapi(summary = "Get Token Pricing Profile")]
#[openapi(description = r#"
Fetches token pricing data and returns a token profile
"#)]
pub async fn get_price_profile(
    #[data] pool: PgPool,
    token_id: i32,
    collection_slug: String,
) -> Result<Json<PriceProfile>, Rejection> {
    println!("/get_price_profile/{}/{}", collection_slug, token_id);
    let mut conn = pool.acquire().await.map_err(internal_error)?;

    let token_traits = get_trait_rarities(&mut conn, &collection_slug.to_string(), token_id)
        .await
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();

    let collection = read_collection(&mut conn, &collection_slug)
        .await
        .map_err(internal_error)?;

    PriceProfile::make(
        &mut conn,
        &collection_slug.to_string(),
        token_id,
        token_traits,
        collection.rarity_cutoff,
    )
    .await
    .map(|r| r.into())
    .map_err(internal_error)
}
