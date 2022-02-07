use crate::opensea::{fetchers::*, os_client::OpenseaAPIClient};
use crate::storage::{establish_connection, read::*, write::*, CollectionSmall};
use anyhow::Result;
use chrono::NaiveDateTime;
use governor::{Quota, RateLimiter};
use sqlx::PgConnection;

pub async fn sync_events_loop() -> Result<()> {
    let rate_limiter =
        RateLimiter::direct(Quota::with_period(std::time::Duration::from_secs(600u64)).unwrap());
    loop {
        rate_limiter.until_ready().await;
        let pool = establish_connection().await;
        let mut conn = pool.acquire().await?;

        let collections = read_all_collections(&mut conn).await?;

        for collection in collections {
            sync_collection(&mut conn, &collection, None, None)
                .await
                .unwrap_or_default();
        }
    }
}

pub async fn sync_collection(
    conn: &mut PgConnection,
    collection: &CollectionSmall,
    occurred_after_listings: Option<&NaiveDateTime>,
    occurred_after_sales: Option<&NaiveDateTime>,
) -> Result<()> {
    let client = OpenseaAPIClient::new(1);

    // Sync Collection Floor
    let floor = client
        .get_collection(&collection.slug)
        .await?
        .collection
        .stats
        .floor_price
        .unwrap_or_default();

    update_collection_floor(conn, &collection.slug, floor)
        .await
        .unwrap_or_default();

    let latest_event = match read_latests_listing_for_collection(conn, &collection.slug).await {
        Ok(l) => l,
        Err(_) => return Ok(()),
    };

    // Sync Cancelled Listing
    let cancelled = fetch_cancelled(
        &client,
        &collection.address,
        occurred_after_listings.unwrap_or(&NaiveDateTime::from_timestamp(latest_event as i64, 0)),
    )
    .await
    .unwrap_or_default();

    for event in cancelled {
        if event.asset.is_none() {
            return Ok(());
        }
        if let Err(e) = write_listing(
            conn,
            &collection.slug,
            "cancelled",
            event.clone().asset.unwrap().token_id as i32,
            None,
            event.created_date.timestamp() as i32,
        )
        .await
        {
            log::info!("Error Storing: {} \n {:?}", e, event)
        }
    }

    // Sync Filled Listings
    let successful = fetch_successful(
        &client,
        &collection.address,
        occurred_after_listings.unwrap_or(&NaiveDateTime::from_timestamp(latest_event as i64, 0)),
    )
    .await
    .unwrap_or_default();

    for event in successful {
        if event.asset.is_none() {
            return Ok(());
        }

        if let Err(e) = write_listing(
            conn,
            &collection.slug,
            "successful",
            event.clone().asset.unwrap().token_id as i32,
            None,
            event.created_date.timestamp() as i32,
        )
        .await
        {
            log::info!("Error Storing: {} \n {:?}", e, event)
        }
    }

    // Sync Created Listings
    let created = fetch_created(
        &client,
        &collection.address,
        occurred_after_listings.unwrap_or(&NaiveDateTime::from_timestamp(latest_event as i64, 0)),
    )
    .await
    .unwrap_or_default();

    for event in created {
        if event.asset.is_none() {
            return Ok(());
        }
        if let Err(e) = write_listing(
            conn,
            &collection.slug,
            "created",
            event.clone().asset.unwrap().token_id as i32,
            Some(event.clone().ending_price.unwrap().parse::<f64>().unwrap()),
            event.created_date.timestamp() as i32,
        )
        .await
        {
            log::info!("Error Storing: {} \n {:?}", e, event)
        }
    }

    let latest_sale = match read_latest_sale_for_collection(conn, &collection.slug).await {
        Ok(l) => l,
        Err(_) => return Ok(()),
    };

    // Sync Transfers
    let transfers = fetch_collection_transfers(
        &client,
        &collection.address,
        occurred_after_listings.unwrap_or(&NaiveDateTime::from_timestamp(latest_sale as i64, 0)),
    )
    .await
    .unwrap_or_default();

    for e in &transfers {
        if e.to_account.is_some() {
            let token_id = if e.asset.is_some() {
                e.asset.as_ref().unwrap().token_id as i32
            } else {
                return Ok(());
            };

            let new_owner = if e.to_account.is_some() {
                e.to_account.as_ref().unwrap().address.clone()
            } else {
                return Ok(());
            };
            write_transfer(conn, token_id, new_owner, &collection.slug)
                .await
                .unwrap_or_default();
        }
    }

    // Sync Sales
    let sales = fetch_collection_sales(
        &client,
        &collection.address,
        occurred_after_sales.unwrap_or(&NaiveDateTime::from_timestamp(latest_sale as i64, 0)),
    )
    .await
    .unwrap_or_default();

    for e in &sales {
        if let Some(p) = e.payment_token.clone() {
            if p.symbol == "ETH" {
                write_sale(conn, e, &collection.slug)
                    .await
                    .unwrap_or_default();

                let token_id = if e.asset.is_some() {
                    e.asset.as_ref().unwrap().token_id as i32
                } else {
                    return Ok(());
                };

                let new_owner = if e.winner_account.is_some() {
                    e.winner_account.as_ref().unwrap().address.clone()
                } else {
                    return Ok(());
                };

                write_transfer(conn, token_id, new_owner, &collection.slug)
                    .await
                    .unwrap_or_default();
            }
        }
    }

    Ok(())
}
