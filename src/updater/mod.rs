use crate::opensea::{types::EventsRequest, OpenseaAPIClient};
use crate::storage::{read::*, write::*};
use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgConnection;

pub async fn update_token_listings(conn: &mut PgConnection, collection_slug: &str) -> Result<()> {
    let latest_listing = read_latests_listing_for_collection(conn, collection_slug)
        .await
        .unwrap();

    let collection = read_collection(conn, collection_slug).await.unwrap();

    let client = OpenseaAPIClient::new();

    let req = EventsRequest::new()
        .asset_contract_address(&collection.address)
        .event_type("cancelled")
        .occurred_after(&NaiveDateTime::from_timestamp(latest_listing as i64, 0).to_string())
        .build();

    let cancelled = client.get_events(req).await.unwrap();
    println!("{} Cancelled Listings", cancelled.len());
    for event in cancelled {
        write_listing(
            conn,
            collection_slug,
            event.asset.unwrap().token_id as i32,
            None,
        )
        .await
        .unwrap();
    }

    let req = EventsRequest::new()
        .asset_contract_address(&collection.address)
        .event_type("created")
        .occurred_after(&NaiveDateTime::from_timestamp(latest_listing as i64, 0).to_string())
        .build();

    let created = client.get_events(req).await.unwrap();
    println!("{} Created Listings", created.len());

    for event in created {
        println!("{:?}", event);
        write_listing(
            conn,
            collection_slug,
            event.asset.unwrap().token_id as i32,
            Some(event.ending_price.unwrap().parse::<f64>().unwrap()),
        )
        .await
        .unwrap();
    }

    Ok(())
}
