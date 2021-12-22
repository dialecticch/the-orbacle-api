use crate::from_wei;
use crate::opensea::OpenseaAPIClient;
use crate::storage::read::*;
use anyhow::Result;
use chrono::prelude::Utc;
use sqlx::PgConnection;

use super::rarities::get_trait_rarities;
use super::sales::*;

pub async fn get_token_listing(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<Option<f64>> {
    let client = OpenseaAPIClient::new();
    let asset = client
        .fetch_token_ids(collection_slug, vec![token_id as u64])
        .await?[0]
        .clone();

    Ok(if asset.sell_orders.is_some() {
        Some(from_wei(asset.sell_orders.unwrap()[0].current_price))
    } else {
        None
    })
}
pub async fn get_trait_listing(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
) -> Result<Vec<(i32, f64)>> {
    let assets_with_trait = read_assets_with_trait(conn, collection_slug, trait_name)
        .await
        .unwrap();

    let ids: Vec<_> = assets_with_trait
        .into_iter()
        .map(|a| a.token_id as u64)
        .collect();

    let client = OpenseaAPIClient::new();

    let mut all_assets: Vec<_> = client.fetch_token_ids(collection_slug, ids).await?;

    all_assets = all_assets
        .into_iter()
        .filter(|a| a.sell_orders.is_some())
        .collect::<Vec<_>>();

    all_assets.sort_by(|a, b| {
        a.sell_orders.clone().unwrap()[0]
            .current_price
            .partial_cmp(&b.sell_orders.clone().unwrap()[0].current_price)
            .unwrap()
    });

    Ok(all_assets
        .into_iter()
        .map(|t| {
            (
                t.token_id as i32,
                from_wei(t.sell_orders.clone().unwrap()[0].current_price),
            )
        })
        .collect())
}

pub async fn get_trait_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
) -> Result<Option<(i32, f64)>> {
    let listings = get_trait_listing(conn, collection_slug, trait_name).await?;

    if listings.is_empty() {
        Ok(None)
    } else {
        Ok(Some((listings[0].0, listings[0].1)))
    }
}

pub async fn get_collection_floor(conn: &mut PgConnection, collection_slug: &str) -> Result<f64> {
    let client = OpenseaAPIClient::new();
    let collection = client.get_collection(collection_slug).await?;

    Ok(collection.collection.stats.floor_price)
}

pub async fn get_most_valued_trait_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
    rarity_cap: f64,
) -> Result<(Option<String>, Option<f64>)> {
    let token_traits = get_trait_rarities(conn, collection_slug, token_id)
        .await?
        .into_iter()
        .filter(|(_, r)| r < &rarity_cap)
        .collect::<Vec<_>>();

    let mut highest_floor = (String::default(), 0f64);
    for (trait_name, _) in token_traits {
        let trait_listings = get_trait_listing(conn, collection_slug, &trait_name).await?;
        if !trait_listings.is_empty() {
            if trait_listings[0].1 > highest_floor.1 {
                highest_floor.0 = trait_name.clone();
                highest_floor.1 = trait_listings[0].1;
            }
        }
    }
    Ok(if highest_floor.0 != String::default() {
        (Some(highest_floor.0), Some(highest_floor.1))
    } else {
        (None, None)
    })
}

pub async fn get_rarest_trait_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<(String, i32, f64)> {
    let token_traits = get_trait_rarities(conn, collection_slug, token_id).await?;

    let listings = get_trait_listing(conn, collection_slug, &token_traits[0].0).await?;
    Ok((token_traits[0].0.clone(), listings[0].0, listings[0].1))
}

pub async fn get_rarity_weighted_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
    rarity_cap: f64,
) -> Result<f64> {
    let token_traits = get_trait_rarities(conn, collection_slug, token_id)
        .await?
        .into_iter()
        .filter(|(_, r)| r < &rarity_cap)
        .collect::<Vec<_>>();

    if token_traits.len() == 0 {
        return Ok(get_rarest_trait_floor(conn, collection_slug, token_id)
            .await?
            .2);
    }

    let mut floors = vec![];
    for (trait_name, raritiy) in token_traits {
        let trait_listings = get_trait_listing(conn, collection_slug, &trait_name).await?;
        if !trait_listings.is_empty() {
            floors.push((trait_listings[0].0, trait_listings[0].1, raritiy));
        }
    }

    floors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut price = floors[0].1;
    for (idx, f) in floors.iter().enumerate().skip(1) {
        price += f.1 / (2f64 * f.2 / floors[idx - 1].2)
    }
    Ok(price)
}

pub async fn get_last_sale_price(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<Option<f64>> {
    let asset_sales = get_asset_sales(conn, collection_slug, token_id).await?;

    if !asset_sales.is_empty() {
        Ok(Some(asset_sales.last().unwrap().1))
    } else {
        Ok(None)
    }
}

pub async fn get_most_valued_trait_last_sale_avg(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
    rarity_cap: f64,
    nr: Option<usize>,
) -> Result<Option<f64>> {
    let token_traits = get_trait_rarities(conn, collection_slug, token_id)
        .await?
        .into_iter()
        .filter(|(_, r)| r < &rarity_cap)
        .collect::<Vec<_>>();

    let mut highest_sale = 0f64;
    for (trait_name, _) in token_traits {
        let trait_sales =
            get_average_trait_sales_nr(conn, collection_slug, &trait_name, nr).await?;
        if !trait_sales.is_none() {
            if trait_sales.unwrap() > highest_sale {
                highest_sale = trait_sales.unwrap();
            }
        }
    }
    Ok(if highest_sale != 0f64 {
        Some(highest_sale)
    } else {
        None
    })
}

pub async fn get_last_sale_relative_to_collection_avg(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<Option<f64>> {
    let asset_sales = get_asset_sales(conn, collection_slug, token_id).await?;

    if !asset_sales.is_empty() {
        let last_sale = asset_sales.last().unwrap();
        let avg_at_sale =
            match get_average_collection_sales_at_ts(conn, collection_slug, &last_sale.2).await? {
                Some(v) => v,
                None => return Ok(None),
            };

        let avg_now = match get_average_collection_sales_at_ts(
            conn,
            collection_slug,
            &Utc::now().naive_utc(),
        )
        .await?
        {
            Some(v) => v,
            None => return Ok(None),
        };

        Ok(Some((last_sale.1 / avg_at_sale) * avg_now))
    } else {
        Ok(None)
    }
}

pub async fn get_last_sale_relative_to_mvt_avg(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
    rarity_cap: f64,
) -> Result<Option<f64>> {
    let token_traits = get_trait_rarities(conn, collection_slug, token_id)
        .await?
        .into_iter()
        .filter(|(_, r)| r < &rarity_cap)
        .collect::<Vec<_>>();

    let mut highest_sale = 0f64;
    for (trait_name, _) in token_traits {
        let trait_sales =
            get_last_sale_relative_to_trait_avg(conn, collection_slug, &trait_name, token_id)
                .await?;
        if !trait_sales.is_none() {
            if trait_sales.unwrap() > highest_sale {
                highest_sale = trait_sales.unwrap();
            }
        }
    }
    Ok(if highest_sale != 0f64 {
        Some(highest_sale)
    } else {
        None
    })
}

pub async fn get_last_sale_relative_to_trait_avg(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
    token_id: i32,
) -> Result<Option<f64>> {
    let asset_sales = get_asset_sales(conn, collection_slug, token_id).await?;

    if !asset_sales.is_empty() {
        let last_sale = asset_sales.last().unwrap();
        let avg_at_sale =
            match get_average_trait_sales_at_ts(conn, collection_slug, trait_name, &last_sale.2)
                .await?
            {
                Some(v) => v,
                None => return Ok(None),
            };
        let avg_now = match get_average_trait_sales_at_ts(
            conn,
            collection_slug,
            trait_name,
            &Utc::now().naive_utc(),
        )
        .await?
        {
            Some(v) => v,
            None => return Ok(None),
        };

        Ok(Some((last_sale.1 / avg_at_sale) * avg_now))
    } else {
        Ok(None)
    }
}
