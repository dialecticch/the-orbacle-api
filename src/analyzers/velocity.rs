use super::sales::*;
use crate::analyzers::prices::get_most_valued_trait_floor;
use crate::analyzers::rarities::*;
use anyhow::Result;
use chrono::prelude::Utc;
use chrono::Duration;
use sqlx::PgConnection;

pub async fn get_sale_frequency_trait(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
    days_back: usize,
) -> Result<f64> {
    let days_ago = Utc::now().naive_utc() - Duration::days(days_back as i64);

    let nr_sales = get_trait_sales(conn, collection_slug, trait_name)
        .await
        .unwrap()
        .into_iter()
        .filter(|s| s.2 > days_ago)
        .count();

    Ok(days_back as f64 / nr_sales as f64)
}

pub async fn get_lowest_sale_frequency(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
    days_back: usize,
) -> Result<(String, f64)> {
    let token_traits = get_trait_rarities(conn, collection_slug, token_id).await?;

    let mut lowest_frequency = (String::default(), f64::MIN);
    for (trait_name, _) in token_traits {
        let frequency =
            get_sale_frequency_trait(conn, collection_slug, &trait_name, days_back).await?;
        if frequency > lowest_frequency.1 {
            lowest_frequency.1 = frequency;
            lowest_frequency.0 = trait_name.clone();
        }
    }
    Ok(lowest_frequency)
}

pub async fn get_avg_sale_frequency(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
    days_back: usize,
) -> Result<f64> {
    let token_traits = get_trait_rarities(conn, collection_slug, token_id).await?;

    let mut cumulative_frequency = 0f64;
    for (trait_name, _) in &token_traits {
        let frequency =
            get_sale_frequency_trait(conn, collection_slug, trait_name, days_back).await?;
        cumulative_frequency += frequency as f64;
    }
    Ok(cumulative_frequency / token_traits.len() as f64)
}

pub async fn get_mvt_sale_frequency(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
    days_back: usize,
) -> Result<f64> {
    let most_valuable_trait = get_most_valued_trait_floor(conn, collection_slug, token_id, 0.01)
        .await?
        .0;
    let frequency = get_sale_frequency_trait(
        conn,
        collection_slug,
        &most_valuable_trait.unwrap(),
        days_back,
    )
    .await?;

    Ok(frequency)
}
