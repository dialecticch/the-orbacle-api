use crate::from_wei;
use crate::storage::read::*;
use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgConnection;

pub async fn get_trait_sales(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
) -> Result<Vec<(i32, f64, NaiveDateTime)>> {
    let mut all_sales = read_sales_for_trait(conn, collection_slug, trait_name)
        .await
        .unwrap();

    all_sales.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(all_sales
        .into_iter()
        .map(|t| {
            (
                t.token_id as i32,
                from_wei(t.price),
                NaiveDateTime::from_timestamp(t.timestamp as i64, 0),
            )
        })
        .collect())
}

pub async fn get_asset_sales(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<Vec<(i32, f64, NaiveDateTime)>> {
    let mut all_sales = read_sales_for_asset(conn, collection_slug, token_id)
        .await
        .unwrap();

    all_sales.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(all_sales
        .into_iter()
        .map(|t| {
            (
                t.token_id as i32,
                from_wei(t.price),
                NaiveDateTime::from_timestamp(t.timestamp as i64, 0),
            )
        })
        .collect())
}

pub async fn get_average_trait_sales_nr(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
    nr: Option<usize>,
) -> Result<Option<f64>> {
    let sale_history = get_trait_sales(conn, collection_slug, trait_name).await?;
    let count = nr.unwrap_or(sale_history.len());

    if sale_history.len() < count || sale_history.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(
            sale_history
                .iter()
                .skip(sale_history.len() - count)
                .map(|s| s.1)
                .sum::<f64>()
                / count as f64,
        ))
    }
}

pub async fn get_average_collection_sales_at_ts(
    conn: &mut PgConnection,
    collection_slug: &str,
    ts: &NaiveDateTime,
) -> Result<Option<f64>> {
    read_avg_price_collection_at_ts(conn, collection_slug, ts).await
}

pub async fn get_average_trait_sales_at_ts(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
    ts: &NaiveDateTime,
) -> Result<Option<f64>> {
    read_avg_price_trait_at_ts(conn, collection_slug, trait_name, ts).await
}
