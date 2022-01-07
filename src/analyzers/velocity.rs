use super::sales::*;
use crate::analyzers::rarities::*;
use anyhow::Result;
use chrono::prelude::Utc;
use chrono::Duration;
use sqlx::PgConnection;

pub async fn get_sale_count_trait(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
    days_back: usize,
) -> Result<usize> {
    let days_ago = Utc::now().naive_utc() - Duration::days(days_back as i64);

    let nr_sales = get_trait_sales(conn, collection_slug, trait_name)
        .await
        .unwrap()
        .into_iter()
        .filter(|s| s.time > days_ago)
        .count();

    Ok(nr_sales as usize)
}

pub async fn get_lowest_sale_count(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
    days_back: usize,
) -> Result<(String, usize)> {
    let token_traits = get_trait_rarities(conn, collection_slug, token_id).await?;

    let mut lowest_frequency = (String::default(), usize::MIN);
    for t in token_traits {
        let frequency = get_sale_count_trait(conn, collection_slug, &t.trait_id, days_back).await?;
        if frequency > lowest_frequency.1 {
            lowest_frequency.1 = frequency;
            lowest_frequency.0 = t.trait_id.clone();
        }
    }
    Ok(lowest_frequency)
}

pub async fn get_avg_sale_count(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
    days_back: usize,
) -> Result<f64> {
    let token_traits = get_trait_rarities(conn, collection_slug, token_id).await?;

    let mut cumulative_frequency = 0f64;
    for t in &token_traits {
        let frequency = get_sale_count_trait(conn, collection_slug, &t.trait_id, days_back).await?;
        cumulative_frequency += frequency as f64;
    }
    Ok(cumulative_frequency / token_traits.len() as f64)
}
