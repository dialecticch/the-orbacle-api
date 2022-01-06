use super::super::errors::{internal_error, ServiceError};
use crate::analyzers::rarities::get_collection_avg_trait_rarity;
use crate::opensea::types::AssetsRequest;
use crate::opensea::OpenseaAPIClient;
use crate::storage::preprocess;
use crate::storage::write::*;
use crate::updater::*;
use anyhow::Result;
use chrono::{Duration, Utc};
use rweb::*;
use sqlx::{PgConnection, PgPool};

#[derive(serde::Deserialize, rweb::Schema)]
pub struct NewCollectionBody {
    pub collection_slug: String,
    pub total_supply_expected: usize,
    pub rarity_cutoff_multiplier: f64,
    pub ignored_trait_types: Vec<String>,
    pub ignored_trait_values: Vec<String>,
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
    let mut conn = pool.acquire().await.map_err(internal_error)?;

    if key != dotenv::var("ADMIN").unwrap() {
        return Err(warp::reject::custom(ServiceError::Unauthorized));
    }

    _store_collection(
        &mut conn,
        &req.collection_slug,
        req.total_supply_expected,
        req.rarity_cutoff_multiplier,
        req.ignored_trait_types,
        req.ignored_trait_values,
    )
    .await
    .map_err(internal_error)?;
    Ok(().into())
}

async fn _store_collection(
    conn: &mut PgConnection,
    collection_slug: &str,
    total_supply: usize,
    multiplier: f64,
    ignored_trait_types: Vec<String>,
    ignored_trait_values: Vec<String>,
) -> Result<()> {
    let client = OpenseaAPIClient::new(1);
    let mut collection = client.get_collection(collection_slug).await?;

    let len_bef = collection.collection.traits.len();

    collection.collection.traits = collection
        .collection
        .traits
        .into_iter()
        .filter(|(t, _)| !ignored_trait_types.contains(&t.to_lowercase()))
        .collect();

    let collection_avg_trait_rarity = get_collection_avg_trait_rarity(&collection.collection)?;

    write_collection(
        conn,
        &collection.collection,
        collection_avg_trait_rarity,
        multiplier,
        ignored_trait_types,
        ignored_trait_values,
    )
    .await
    .unwrap_or_default();

    write_traits(conn, &collection.collection)
        .await
        .unwrap_or_default();
    println!("  Stored traits stats!");

    println!("  Fetching assets...");

    let req = AssetsRequest::new()
        .collection(collection_slug)
        .expected(total_supply)
        .build();

    let all_assets = client.get_assets(req).await?;

    println!("  Storing {} assets...", all_assets.len());

    let processed = preprocess::process_assets(conn, all_assets.clone(), collection_slug).await?;

    for a in &processed {
        write_asset(conn, a).await.unwrap();
    }

    println!("  Stored {} assets!", all_assets.len());

    println!("  Storing listings...");

    for a in &all_assets {
        if a.sell_orders.is_some() {
            write_listing(
                conn,
                collection_slug,
                "sell_order",
                a.token_id as i32,
                Some(a.sell_orders.clone().unwrap()[0].current_price),
                a.sell_orders.clone().unwrap()[0].created_date.timestamp() as i32,
            )
            .await
            .unwrap();
        } else {
            write_listing(
                conn,
                collection_slug,
                "sell_order",
                a.token_id as i32,
                None,
                Utc::now().timestamp() as i32,
            )
            .await
            .unwrap();
        }
    }
    println!("  Stored {} Listings!", all_assets.len());

    println!("  Fetching events...");

    let now = Utc::now();

    fetch_collection_listings(
        conn,
        collection_slug,
        &(now - Duration::days(14)).naive_utc(),
    )
    .await
    .unwrap();

    fetch_collection_sales(conn, collection_slug, None)
        .await
        .unwrap();

    Ok(())
}
