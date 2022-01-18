use super::super::errors::internal_error;
use crate::analyzers::prices::get_most_valued_trait_floor;
use crate::analyzers::rarities::get_trait_rarities;
use crate::profiles::token::collection_profile::CollectionProfile;
use crate::profiles::token::price_profile::PriceProfile;
use crate::profiles::token::token_profile::TokenProfile;
use crate::profiles::token::wallet_profile::WalletProfile;
use crate::storage::{
    read::{read_all_collections, read_collection},
    CollectionSmall,
};
use anyhow::Result;
use cached::proc_macro::cached;
use rweb::*;
use sqlx::{PgConnection, PgPool};

#[get("/profile/{collection_slug}/{token_id}")]
#[openapi(tags("Token"))]
#[openapi(summary = "Get a profile for token")]
#[openapi(description = r#"
    Gets token data and returns the full profile for token and collection
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
    size = 25,
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
#[openapi(tags("Token"))]
#[openapi(summary = "Get pricing for token")]
#[openapi(description = r#"
    Gets token data and returns only the pricing for the token
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
    size = 25,
    result = true,
    key = "String",
    convert = r#"{ format!("{}{}", collection_slug, token_id) }"#
)]
async fn _get_price_profile(
    conn: &mut PgConnection,
    collection_slug: String,
    token_id: i32,
) -> Result<PriceProfile> {
    let collection = read_collection(conn, &collection_slug).await?;

    let token_traits = get_trait_rarities(conn, &collection_slug, token_id).await?;

    let rarest_trait = token_traits[0].trait_id.clone();

    let most_valuable_trait = get_most_valued_trait_floor(
        conn,
        &collection_slug,
        token_traits.clone(),
        collection.rarity_cutoff,
    )
    .await?;

    PriceProfile::make(
        conn,
        &collection_slug.to_string(),
        token_id,
        token_traits,
        &rarest_trait,
        &most_valuable_trait,
        collection.rarity_cutoff,
    )
    .await
}

#[get("/collection/{collection_slug}")]
#[openapi(tags("Collection"))]
#[openapi(summary = "Get Profile for collection")]
#[openapi(description = r#"
Gets the collection profile for given collection_slug
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
#[openapi(tags("Collection"))]
#[openapi(summary = "Get all collection names")]
#[openapi(description = r#"
Gets a list of all supported Collections
"#)]
pub async fn get_all_collections(
    #[data] pool: PgPool,
) -> Result<Json<Vec<CollectionSmall>>, Rejection> {
    println!("/get_all_collections/");
    let mut conn = pool.acquire().await.map_err(internal_error)?;

    read_all_collections(&mut conn)
        .await
        .map(|r| r.into())
        .map_err(internal_error)
}

#[get("/wallet/{collection_slug}/{wallet}")]
#[openapi(tags("Wallet"))]
#[openapi(summary = "Get Wallet profile")]
#[openapi(description = r#"
Gets all pricings for tokens in collection in wallet and get total amounts
"#)]
pub async fn get_wallet_profile(
    #[data] pool: PgPool,
    wallet: String,
    collection_slug: String,
) -> Result<Json<WalletProfile>, Rejection> {
    println!("/get_wallet/{}/{}", collection_slug, wallet);

    _get_wallet_profile(pool, collection_slug, wallet)
        .await
        .map(|r| r.into())
        .map_err(internal_error)
}

#[cached(
    size = 25,
    result = true,
    key = "String",
    convert = r#"{ format!("{}{}", collection_slug, wallet) }"#
)]
pub async fn _get_wallet_profile(
    pool: PgPool,
    collection_slug: String,
    wallet: String,
) -> Result<WalletProfile> {
    WalletProfile::make(pool, &collection_slug, &wallet).await
}
