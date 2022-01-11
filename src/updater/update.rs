use super::*;
use crate::storage::{establish_connection, read::*};
use anyhow::Result;
use chrono::NaiveDateTime;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    RateLimiter,
};

pub async fn update_db(
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
) -> Result<()> {
    loop {
        rate_limiter.until_ready().await;
        let pool = establish_connection().await;
        let mut conn = pool.acquire().await?;

        let collections = read_all_collections(&mut conn).await?;

        for collection in collections {
            fetch_collection_floor(&mut conn, &collection.slug)
                .await
                .unwrap_or_default();

            let latest_listing =
                match read_latests_listing_for_collection(&mut conn, &collection.slug).await {
                    Ok(l) => l,
                    Err(_) => continue,
                };

            fetch_collection_listings(
                &mut conn,
                &collection.slug,
                &NaiveDateTime::from_timestamp(latest_listing as i64, 0),
            )
            .await
            .unwrap_or_default();

            let latest_sale =
                match read_latest_sale_for_collection(&mut conn, &collection.slug).await {
                    Ok(l) => l,
                    Err(_) => continue,
                };

            fetch_collection_sales(
                &mut conn,
                &collection.slug,
                Some(NaiveDateTime::from_timestamp(latest_sale as i64, 0)),
            )
            .await
            .unwrap_or_default();
        }
    }
}
