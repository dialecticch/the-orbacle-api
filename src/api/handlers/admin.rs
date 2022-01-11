use super::super::errors::{internal_error, ServiceError};
use crate::analyzers::rarities::get_collection_avg_trait_rarity;
use crate::opensea::types::AssetsRequest;
use crate::opensea::{types::Trait, OpenseaAPIClient};
use crate::storage::delete::*;
use crate::storage::preprocess;
use crate::storage::write::*;
use crate::storage::Trait as StorageTrait;
use crate::updater::*;
use anyhow::Result;
use chrono::{Duration, Utc};
use rweb::*;
use sqlx::{PgConnection, PgPool};
use std::collections::HashSet;

#[derive(serde::Deserialize, rweb::Schema)]
pub struct NewCollectionBody {
    pub collection_slug: String,
    pub total_supply_expected: usize,
    pub rarity_cutoff_multiplier: f64,
    pub ignored_trait_types_rarity: Vec<String>,
    pub ignored_trait_types_overlap: Vec<String>,
}

#[post("/admin/collection/")]
#[openapi(tags("Admin"))]
#[openapi(summary = "Add a new collection")]
#[openapi(description = r#"
Fetches and stores collection data
"#)]
pub async fn new_collection(
    #[data] pool: PgPool,
    #[header = "x-api-key"] key: String,
    body: rweb::Json<NewCollectionBody>,
) -> Result<Json<()>, Rejection> {
    let req: NewCollectionBody = body.into_inner();
    println!("/new_collection/{}/{}", key, req.collection_slug);

    if key != dotenv::var("ADMIN_API_KEY").unwrap() {
        return Err(warp::reject::custom(ServiceError::Unauthorized));
    }
    tokio::task::spawn(_store_collection(
        pool,
        req.collection_slug.clone(),
        req.total_supply_expected,
        req.rarity_cutoff_multiplier,
        req.ignored_trait_types_rarity
            .into_iter()
            .map(|t| t.to_lowercase())
            .collect(),
        req.ignored_trait_types_overlap,
    ));
    Ok(().into())
}

async fn _store_collection(
    pool: PgPool,
    collection_slug: String,
    total_supply: usize,
    multiplier: f64,
    ignored_trait_types_rarity: Vec<String>,
    ignored_trait_types_overlap: Vec<String>,
) -> Result<()> {
    let mut conn = pool.acquire().await?;
    let client = OpenseaAPIClient::new(1);
    let collection = client.get_collection(&collection_slug).await?;

    let req = AssetsRequest::new()
        .collection(&collection_slug)
        .expected(total_supply)
        .build();

    let all_assets = client.get_assets(req).await?;

    let traits_all = all_assets
        .clone()
        .iter()
        .map(|a| a.traits.clone())
        .flatten()
        .flatten()
        .collect::<Vec<_>>();

    let traits_filtered: HashSet<Trait> = traits_all
        .into_iter()
        .filter(|t| t.trait_count.is_some())
        .filter(|t| !ignored_trait_types_rarity.contains(&t.trait_type.to_lowercase()))
        .collect();

    let traits: Vec<StorageTrait> = traits_filtered
        .into_iter()
        .map(|t| StorageTrait {
            collection_slug: collection_slug.to_lowercase(),
            trait_id: format!(
                "{}:{}",
                &t.trait_type.to_lowercase(),
                &t.value.to_lowercase()
            ),
            trait_type: t.trait_type.to_lowercase(),
            trait_name: t.value.to_lowercase(),
            trait_count: t.trait_count.unwrap() as i32,
        })
        .collect::<Vec<StorageTrait>>();

    let collection_avg_trait_rarity = get_collection_avg_trait_rarity(&traits)?;

    write_traits(&mut conn, traits).await.unwrap_or_default();

    write_collection(
        &mut conn,
        &collection.collection,
        collection_avg_trait_rarity,
        multiplier,
        ignored_trait_types_rarity.clone(),
        ignored_trait_types_overlap.clone(),
    )
    .await
    .unwrap_or_default();

    let processed = preprocess::process_assets(all_assets.clone(), &collection_slug).await?;

    for a in &processed {
        write_asset(&mut conn, a).await.unwrap();
    }

    let updated =
        preprocess::generate_overlaps(processed, &collection_slug, &ignored_trait_types_overlap)
            .await?;

    for a in &updated {
        update_asset_overlap(&mut conn, a).await.unwrap();
    }

    for a in &all_assets {
        if a.sell_orders.is_some() {
            write_listing(
                &mut conn,
                &collection_slug,
                "sell_order",
                a.token_id as i32,
                Some(a.sell_orders.clone().unwrap()[0].current_price),
                a.sell_orders.clone().unwrap()[0].created_date.timestamp() as i32,
            )
            .await
            .unwrap();
        } else {
            write_listing(
                &mut conn,
                &collection_slug,
                "sell_order",
                a.token_id as i32,
                None,
                Utc::now().timestamp() as i32,
            )
            .await
            .unwrap();
        }
    }

    let now = Utc::now();

    fetch_collection_listings(
        &mut conn,
        &collection_slug,
        &(now - Duration::days(30)).naive_utc(),
    )
    .await
    .unwrap();

    fetch_collection_sales(
        &mut conn,
        &collection_slug,
        Some(collection.collection.primary_asset_contracts[0].created_date),
    )
    .await
    .unwrap();

    println!("Done storing!");

    Ok(())
}

#[patch("/admin/collection/")]
#[openapi(tags("Admin"))]
#[openapi(summary = "Update values in a new collection")]
#[openapi(description = r#"
    Update ignored trait types and cutoff multiplier
"#)]
pub async fn update_collection(
    #[data] pool: PgPool,
    #[header = "x-api-key"] key: String,
    body: rweb::Json<NewCollectionBody>,
) -> Result<Json<()>, Rejection> {
    let req: NewCollectionBody = body.into_inner();
    println!("/update_collection/{}/{}", key, req.collection_slug);

    if key != dotenv::var("ADMIN_API_KEY").unwrap() {
        return Err(warp::reject::custom(ServiceError::Unauthorized));
    }
    tokio::task::spawn(_update_collection(
        pool,
        req.collection_slug.clone(),
        req.rarity_cutoff_multiplier,
        req.ignored_trait_types_rarity,
        req.ignored_trait_types_overlap,
    ));
    Ok(().into())
}

async fn _update_collection(
    pool: PgPool,
    collection_slug: String,
    multiplier: f64,
    ignored_trait_types_rarity: Vec<String>,
    ignored_trait_types_overlap: Vec<String>,
) -> Result<()> {
    let mut conn = pool.acquire().await?;
    let client = OpenseaAPIClient::new(1);
    let collection = client.get_collection(&collection_slug).await?;

    let total_supply = collection.collection.stats.total_supply;

    let req = AssetsRequest::new()
        .collection(&collection_slug)
        .expected(total_supply as usize)
        .build();

    let all_assets = client.get_assets(req).await?;

    let traits = all_assets
        .clone()
        .iter()
        .map(|a| a.traits.clone())
        .flatten()
        .flatten()
        .filter(|t| t.trait_count.is_some())
        .filter(|t| !ignored_trait_types_rarity.contains(&t.trait_type.to_lowercase()))
        .map(|t| StorageTrait {
            collection_slug: collection_slug.to_lowercase(),
            trait_id: format!(
                "{}:{}",
                &t.trait_type.to_lowercase(),
                &t.value.to_lowercase()
            ),
            trait_type: t.trait_type.to_lowercase(),
            trait_name: t.value.to_lowercase(),
            trait_count: t.trait_count.unwrap() as i32,
        })
        .collect::<Vec<StorageTrait>>();

    let collection_avg_trait_rarity = get_collection_avg_trait_rarity(&traits)?;

    update_collection_info(
        &mut conn,
        &collection.collection.slug,
        total_supply,
        ignored_trait_types_rarity,
        ignored_trait_types_overlap,
        (collection_avg_trait_rarity * multiplier) / total_supply,
    )
    .await
    .unwrap_or_default();

    update_traits(&mut conn, &collection_slug, traits)
        .await
        .unwrap_or_default();

    println!("Done updating!");

    Ok(())
}

#[delete("/admin/collection/{collection_slug}")]
#[openapi(tags("Admin"))]
#[openapi(summary = "Delete all collection data")]
#[openapi(description = r#"
Deletes all data for this collection from system, including Assets and Traits
"#)]
pub async fn delete_collection(
    #[data] pool: PgPool,
    #[header = "x-api-key"] key: String,
    collection_slug: String,
) -> Result<Json<()>, Rejection> {
    println!("/delete_collection/{}/{}", key, collection_slug);
    let mut conn = pool.acquire().await.map_err(internal_error)?;

    if key != dotenv::var("ADMIN_API_KEY").unwrap() {
        return Err(warp::reject::custom(ServiceError::Unauthorized));
    }

    _delete_collection(&mut conn, &collection_slug)
        .await
        .map_err(internal_error)?;
    Ok(().into())
}

async fn _delete_collection(conn: &mut PgConnection, collection_slug: &str) -> Result<()> {
    purge_collection(conn, collection_slug)
        .await
        .unwrap_or_default();
    Ok(())
}
