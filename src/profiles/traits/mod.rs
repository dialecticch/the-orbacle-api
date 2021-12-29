use crate::from_wei;
use crate::storage::{read::read_trait_listings_at_ts, Listing};
use anyhow::Result;
use chrono::Utc;
use sqlx::PgConnection;
pub mod trait_profile;
use std::collections::HashMap;

static DAY: i32 = 86400;

pub async fn get_daily_trait_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
) -> Result<Vec<f64>> {
    let ts = Utc::now().timestamp() as i32;

    let listings = read_trait_listings_at_ts(conn, collection_slug, trait_name, ts).await?;

    let mut token_map = HashMap::<i32, Vec<Listing>>::new();
    let ids = listings.iter().map(|l| l.token_id).collect::<Vec<i32>>();
    println!("{:?}", ids.len());
    for i in &ids {
        token_map.insert(
            *i,
            listings
                .iter()
                .filter(|t| t.token_id == *i)
                .cloned()
                .collect::<Vec<_>>(),
        );
    }
    let mut daily = vec![];

    for day in 0..30 {
        let mut min = f64::MAX;
        for i in &ids {
            let l = token_map.get(i).unwrap();
            let mut latest = l
                .iter()
                .filter(|t| t.timestamp < ts - DAY * day)
                .collect::<Vec<_>>();

            latest.sort_by(|a, b| {
                a.price
                    .unwrap_or(f64::MAX)
                    .partial_cmp(&b.price.unwrap_or(f64::MAX))
                    .unwrap()
            });

            if !latest.is_empty() && latest[0].price.unwrap_or(f64::MAX) < min {
                min = from_wei(latest[0].price.unwrap_or(f64::MAX));
            }
        }
        daily.push(min);
    }

    //daily.reverse();

    Ok(daily)
}
