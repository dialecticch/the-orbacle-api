use anyhow::Result;
use local::opensea::types::{AssetsRequest, EventsRequest};
use local::opensea::OpenseaAPIClient;
use local::storage::establish_connection;
use local::storage::write::*;

static COLLECTION: &str = "forgottenruneswizardscult";

#[tokio::main]
pub async fn main() -> Result<()> {
    let pool = establish_connection().await;
    let mut conn = pool.acquire().await?;

    let client = OpenseaAPIClient::new();
    let collection = client.get_collection(COLLECTION).await?;

    //write_collection(&mut conn, &collection.collection).await?;
    println!("Stored collection!");

    //write_traits(&mut conn, &collection.collection).await?;
    println!("Stored traits!");

    let size = collection.collection.stats.total_supply as usize;

    let req = AssetsRequest::new()
        .collection(COLLECTION)
        .expected(size)
        .build();

    //let all_assets = client.get_assets(req).await?;

    // for a in &all_assets {
    //     write_asset(&mut conn, a, COLLECTION)
    //         .await
    //         .unwrap_or_default();
    // }
    // println!("Stored {} assets!", all_assets.len());

    let req = EventsRequest::new()
        .asset_contract_address(&collection.collection.primary_asset_contracts[0].address)
        .event_type("successful")
        //.occurred_after("2021-12-21T04:00:00")
        .expected(collection.collection.stats.total_sales as usize)
        .build();

    let all_events = client.get_events(req).await?;
    println!("Get Events {}!", all_events.len());

    Ok(())
}
