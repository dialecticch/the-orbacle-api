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

        let collections = read_all_collections(&mut conn).await.unwrap();

        for collection_slug in collections {
            fetch_collection_floor(&mut conn, &collection_slug)
                .await
                .unwrap();

            let latest_listing = read_latests_listing_for_collection(&mut conn, &collection_slug)
                .await
                .unwrap();

            fetch_collection_listings(
                &mut conn,
                &collection_slug,
                &NaiveDateTime::from_timestamp(latest_listing as i64, 0),
            )
            .await
            .unwrap();

            let latest_sale = read_latest_sale_for_collection(&mut conn, &collection_slug)
                .await
                .unwrap();

            fetch_collection_sales(
                &mut conn,
                &collection_slug,
                Some(NaiveDateTime::from_timestamp(latest_sale as i64, 0)),
            )
            .await
            .unwrap();
        }
    }
}
