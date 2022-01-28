use super::{
    os_client::OpenseaAPIClient,
    types::{Event, EventsRequest},
};
use anyhow::Result;
use chrono::NaiveDateTime;

pub async fn fetch_cancelled(
    client: &OpenseaAPIClient,
    address: &str,
    occurred_after: &NaiveDateTime,
) -> Result<Vec<Event>> {
    let req = EventsRequest::new()
        .asset_contract_address(address)
        .event_type("cancelled")
        .occurred_after(occurred_after)
        .chunk_size(7)
        .build();

    client.get_events(req).await
}

pub async fn fetch_successful(
    client: &OpenseaAPIClient,
    address: &str,
    occurred_after: &NaiveDateTime,
) -> Result<Vec<Event>> {
    let req = EventsRequest::new()
        .asset_contract_address(address)
        .event_type("successful")
        .occurred_after(occurred_after)
        .chunk_size(7)
        .build();

    client.get_events(req).await
}

pub async fn fetch_created(
    client: &OpenseaAPIClient,
    address: &str,
    occurred_after: &NaiveDateTime,
) -> Result<Vec<Event>> {
    let req = EventsRequest::new()
        .asset_contract_address(address)
        .event_type("created")
        .occurred_after(occurred_after)
        .chunk_size(7)
        .build();

    client.get_events(req).await
}

pub async fn fetch_collection_sales(
    client: &OpenseaAPIClient,
    address: &str,
    occurred_after: &NaiveDateTime,
) -> Result<Vec<Event>> {
    let req = EventsRequest::new()
        .asset_contract_address(address)
        .event_type("successful")
        .chunk_size(7)
        .occurred_after(occurred_after)
        .build();

    let mut sales = client.get_events(req).await.unwrap();

    sales.sort_by(|a, b| a.created_date.cmp(&b.created_date));

    Ok(sales)
}

pub async fn fetch_collection_transfers(
    client: &OpenseaAPIClient,
    address: &str,
    occurred_after: &NaiveDateTime,
) -> Result<Vec<Event>> {
    let req = EventsRequest::new()
        .asset_contract_address(address)
        .event_type("transfer")
        .chunk_size(7)
        .occurred_after(occurred_after)
        .build();

    let mut transfers = client.get_events(req).await.unwrap();

    transfers.sort_by(|a, b| a.created_date.cmp(&b.created_date));

    Ok(transfers)
}
