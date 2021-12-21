use crate::from_wei;
use crate::opensea::OpenseaAPIClient;
use crate::storage::read::*;
use anyhow::Result;
use sqlx::PgConnection;

use super::rarities::get_trait_rarities;

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

pub async fn get_most_valued_trait_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
    rarity_cap: f64,
) -> Result<(String, i32, f64)> {
    let token_traits = get_trait_rarities(conn, collection_slug, token_id)
        .await?
        .into_iter()
        .filter(|(_, r)| r < &rarity_cap)
        .collect::<Vec<_>>();

    let mut highest_floor = (String::default(), 0i32, 0f64);
    for (trait_name, _) in token_traits {
        let trait_listings = get_trait_listing(conn, collection_slug, &trait_name).await?;
        if !trait_listings.is_empty() {
            if trait_listings[0].1 > highest_floor.2 {
                highest_floor.0 = trait_name.clone();
                highest_floor.1 = trait_listings[0].0;
                highest_floor.2 = trait_listings[0].1;
            }
        }
    }
    Ok(highest_floor)
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
