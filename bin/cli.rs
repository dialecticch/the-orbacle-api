use anyhow::Result;
use chrono::{Duration, Utc};
use clap::{App, Arg};
use local::analyzers::rarities::get_collection_avg_trait_rarity;
use local::opensea::types::AssetsRequest;
use local::opensea::OpenseaAPIClient;
use local::storage::establish_connection;
use local::storage::preprocess;
use local::storage::read::read_collection;
use local::storage::write::*;
use local::updater::*;

use local::profiles::token::token_profile::TokenProfile;

#[tokio::main]
pub async fn main() {
    let matches = App::new("nft-pricer")
        .version("1.0")
        .about("Does awesome things")
        .arg(
            Arg::with_name("store")
                .short("-s")
                .long("store")
                .value_name("COLLECTION")
                .value_name("TOT_SUPPLY")
                .help("Stores collection data")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("fetch")
                .short("-f")
                .long("fetch")
                .value_name("COLLECTION")
                .value_name("TOKEN_ID")
                .value_name("CUTOFF")
                .help("fetch token data")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("update")
                .short("-u")
                .long("update")
                .value_name("COLLECTION")
                .value_name("CUTOFF")
                .help("update cutoff")
                .takes_value(true),
        )
        .get_matches();

    if let Some(c) = matches.values_of("store") {
        let params = c.into_iter().collect::<Vec<_>>();
        store(
            params[0],
            params[1]
                .parse::<usize>()
                .expect("TOT_SUPPLY was not and number"),
        )
        .await
        .unwrap();
    }

    if let Some(c) = matches.values_of("update") {
        let pool = establish_connection().await;
        let mut conn = pool.acquire().await.unwrap();
        let params = c.into_iter().collect::<Vec<_>>();
        update_collection_rarity_cutoff(
            &mut conn,
            params[0],
            params[1]
                .parse::<f64>()
                .expect("TOKEN_ID was not and float"),
        )
        .await
        .unwrap();
    }

    if let Some(c) = matches.values_of("fetch") {
        let params = c.into_iter().collect::<Vec<_>>();
        let pool = establish_connection().await;
        let mut conn = pool.acquire().await.unwrap();

        println!("Building Asset Profile...");

        let collection = read_collection(&mut conn, params[0]).await.unwrap();

        let profile = TokenProfile::make(
            &mut conn,
            collection,
            params[1]
                .parse::<i32>()
                .expect("TOKEN_ID was not and number"),
        )
        .await
        .unwrap();

        println!("{}", serde_json::to_string_pretty(&profile).unwrap());
    }
}

async fn store(collection_slug: &str, total_supply: usize) -> Result<()> {
    let pool = establish_connection().await;
    let mut conn = pool.acquire().await?;

    let client = OpenseaAPIClient::new(1);
    let collection = client.get_collection(collection_slug).await?;

    let collection_avg_trait_rarity = get_collection_avg_trait_rarity(&collection.collection)?;

    write_collection(
        &mut conn,
        &collection.collection,
        collection_avg_trait_rarity,
    )
    .await
    .unwrap_or_default();
    println!("  Stored collection stats!");

    write_traits(&mut conn, &collection.collection)
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

    let processed =
        preprocess::process_assets(&mut conn, all_assets.clone(), collection_slug).await?;

    for a in &processed {
        write_asset(&mut conn, a).await.unwrap();
    }

    println!("  Stored {} assets!", all_assets.len());

    println!("  Storing listings...");

    for a in &all_assets {
        if a.sell_orders.is_some() {
            write_listing(
                &mut conn,
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
                &mut conn,
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
        &mut conn,
        collection_slug,
        &(now - Duration::days(14)).naive_utc(),
    )
    .await
    .unwrap();

    fetch_collection_sales(&mut conn, collection_slug, None)
        .await
        .unwrap();

    Ok(())
}
