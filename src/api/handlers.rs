use super::errors::internal_error;
use crate::analyzers::rarities::get_trait_rarities;
use crate::profiles::token::collection_profile::CollectionProfile;
use crate::profiles::token::price_profile::PriceProfile;
use crate::profiles::token::token_profile::TokenProfile;
use crate::profiles::token::wallet_profile::WalletProfile;
use crate::storage::read::{read_all_collections, read_collection};
use anyhow::Result;
use cached::proc_macro::cached;
use rweb::*;
use sqlx::{PgConnection, PgPool};

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

    _get_profile(&mut conn, collection_slug, token_id)
        .await
        .map(|r| r.into())
        .map_err(internal_error)
}

#[cached(
    size = 1,
    result = true,
    key = "String",
    convert = r#"{ format!("{}{}", collection_slug, token_id) }"#
)]
async fn _get_profile(
    conn: &mut PgConnection,
    collection_slug: String,
    token_id: i32,
) -> Result<TokenProfile> {
    let collection = read_collection(conn, &collection_slug).await?;

    TokenProfile::make(conn, collection, token_id).await
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

    _get_price_profile(&mut conn, collection_slug, token_id)
        .await
        .map(|r| r.into())
        .map_err(internal_error)
}

#[cached(
    size = 1,
    result = true,
    key = "String",
    convert = r#"{ format!("{}{}", collection_slug, token_id) }"#
)]
async fn _get_price_profile(
    conn: &mut PgConnection,
    collection_slug: String,
    token_id: i32,
) -> Result<PriceProfile> {
    let token_traits = get_trait_rarities(conn, &collection_slug.to_string(), token_id)
        .await
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();

    let collection = read_collection(conn, &collection_slug).await?;

    PriceProfile::make(
        conn,
        &collection_slug.to_string(),
        token_id,
        token_traits,
        collection.rarity_cutoff,
    )
    .await
}

#[get("/collection/{collection_slug}")]
#[openapi(tags("system"))]
#[openapi(summary = "Get Token Pricing Profile")]
#[openapi(description = r#"
Fetches collection stats
"#)]
pub async fn get_collection_profile(
    #[data] pool: PgPool,
    collection_slug: String,
) -> Result<Json<CollectionProfile>, Rejection> {
    println!("/get_collection/{}", collection_slug);
    let mut conn = pool.acquire().await.map_err(internal_error)?;

    CollectionProfile::make(&mut conn, &collection_slug.to_string())
        .await
        .map(|r| r.into())
        .map_err(internal_error)
}

#[get("/collection/")]
#[openapi(tags("system"))]
#[openapi(summary = "Get Token Pricing Profile")]
#[openapi(description = r#"
Fetches collection stats
"#)]
pub async fn get_all_collections(#[data] pool: PgPool) -> Result<Json<Vec<String>>, Rejection> {
    println!("/get_all_collections/");
    let mut conn = pool.acquire().await.map_err(internal_error)?;

    read_all_collections(&mut conn)
        .await
        .map(|r| r.into())
        .map_err(internal_error)
}

#[get("/wallet/{collection_slug}/{wallet}")]
#[openapi(tags("system"))]
#[openapi(summary = "Get Token Pricing Profile")]
#[openapi(description = r#"
Fetches wallet stats
"#)]
pub async fn get_wallet_profile(
    #[data] pool: PgPool,
    wallet: String,
    collection_slug: String,
) -> Result<Json<WalletProfile>, Rejection> {
    println!("/get_wallet/{}/{}", collection_slug, wallet);
    let mut conn = pool.acquire().await.map_err(internal_error)?;

    _get_wallet_profile(&mut conn, collection_slug, wallet)
        .await
        .map(|r| r.into())
        .map_err(internal_error)
}

#[cached(
    size = 1,
    result = true,
    key = "String",
    convert = r#"{ format!("{}{}", collection_slug, wallet) }"#
)]
pub async fn _get_wallet_profile(
    conn: &mut PgConnection,
    collection_slug: String,
    wallet: String,
) -> Result<WalletProfile> {
    WalletProfile::make(conn, &collection_slug, &wallet).await
}
