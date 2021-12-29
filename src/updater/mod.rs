pub mod update;

use crate::opensea::{types::EventsRequest, OpenseaAPIClient};
use crate::storage::{read::*, write::*};
use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgConnection;

pub async fn fetch_collection_listings(
    conn: &mut PgConnection,
    collection_slug: &str,
    occurred_after: &NaiveDateTime,
) -> Result<()> {
    // let latest_listing = read_latests_listing_for_collection(conn, collection_slug)
    //     .await
    //     .unwrap();

    let collection = read_collection(conn, collection_slug).await.unwrap();

    let client = OpenseaAPIClient::new(3);

    let req = EventsRequest::new()
        .asset_contract_address(&collection.address)
        .event_type("cancelled")
        .occurred_after(&occurred_after.to_string())
        .build();

    let cancelled = client.get_events(req).await.unwrap();
    log::info!("{} Cancelled Listings", cancelled.len());

    for event in cancelled {
        if let None = event.asset {
            continue;
        }
        if let Err(e) = write_listing(
            conn,
            collection_slug,
            event.clone().asset.unwrap().token_id as i32,
            None,
            event.created_date.timestamp() as i32,
        )
        .await
        {
            log::info!("Error Storing: {} \n {:?}", e, event)
        }
    }

    let req = EventsRequest::new()
        .asset_contract_address(&collection.address)
        .event_type("successful")
        .occurred_after(&occurred_after.to_string())
        .build();

    let filled = client.get_events(req).await.unwrap();
    log::info!("{} Filled Listings", filled.len());
    for event in filled {
        if let None = event.asset {
            continue;
        }

        if let Err(e) = write_listing(
            conn,
            collection_slug,
            event.clone().asset.unwrap().token_id as i32,
            None,
            event.created_date.timestamp() as i32,
        )
        .await
        {
            log::info!("Error Storing: {} \n {:?}", e, event)
        }
    }

    let req = EventsRequest::new()
        .asset_contract_address(&collection.address)
        .event_type("created")
        .occurred_after(&occurred_after.to_string())
        .build();

    let created = client.get_events(req).await.unwrap();
    log::info!("{} Created Listings", created.len());

    for event in created {
        if let None = event.asset {
            continue;
        }
        if let Err(e) = write_listing(
            conn,
            collection_slug,
            event.clone().asset.unwrap().token_id as i32,
            Some(event.clone().ending_price.unwrap().parse::<f64>().unwrap()),
            event.created_date.timestamp() as i32,
        )
        .await
        {
            log::info!("Error Storing: {} \n {:?}", e, event)
        }
    }

    Ok(())
}

pub async fn fetch_collection_sales(
    conn: &mut PgConnection,
    collection_slug: &str,
    occurred_after: &NaiveDateTime,
) -> Result<()> {
    let collection = read_collection(conn, collection_slug).await.unwrap();

    let client = OpenseaAPIClient::new(3);

    let req = EventsRequest::new()
        .asset_contract_address(&collection.address)
        .event_type("successful")
        .occurred_after(&occurred_after.to_string())
        .build();

    let sales = client.get_events(req).await.unwrap();
    log::info!("{} New Sales", sales.len());
    for e in &sales {
        write_sale(conn, e, collection_slug)
            .await
            .unwrap_or_default();
    }

    Ok(())
}

pub async fn fetch_collection_floor(conn: &mut PgConnection, collection_slug: &str) -> Result<()> {
    let client = OpenseaAPIClient::new(3);
    let collection = client.get_collection(collection_slug).await?;

    update_collection_floor(
        conn,
        collection_slug,
        collection.collection.stats.floor_price,
    )
    .await
    .unwrap_or_default();

    Ok(())
}
