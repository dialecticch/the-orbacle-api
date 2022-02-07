use anyhow::Result;
use chrono::prelude::Utc;
use sqlx::PgConnection;

use super::listings::get_trait_listings;
use super::sales::*;
use super::*;
use crate::storage::read::read_collection;

pub async fn get_trait_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
) -> Result<Option<TraitFloor>> {
    let listings = get_trait_listings(conn, collection_slug, trait_name).await?;

    if listings.is_empty() {
        Ok(None)
    } else {
        Ok(Some(TraitFloor {
            token_id: listings[0].token_id,
            trait_id: trait_name.to_string(),
            floor_price: listings[0].price,
        }))
    }
}

pub async fn get_collection_floor(conn: &mut PgConnection, collection_slug: &str) -> Result<f64> {
    let collection = read_collection(conn, collection_slug).await?;

    Ok(collection.floor_price)
}

pub async fn get_most_valued_trait_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_traits: Vec<TraitRarities>,
    cutoff: f64,
) -> Result<Option<TraitFloor>> {
    let mut token_traits_filtered = token_traits
        .iter()
        .filter(|t| t.rarity < cutoff)
        .cloned()
        .collect::<Vec<_>>();

    // if all traits are above the average rarity
    if token_traits_filtered.is_empty() {
        token_traits_filtered = token_traits.clone();
    }

    let mut highest_floor = TraitFloor::default();
    for t in token_traits_filtered {
        let trait_listings = get_trait_listings(conn, collection_slug, &t.trait_id).await?;
        if !trait_listings.is_empty() && trait_listings[0].price > highest_floor.floor_price {
            highest_floor = TraitFloor {
                trait_id: t.trait_id.clone(),
                token_id: trait_listings[0].token_id,
                floor_price: trait_listings[0].price,
            }
        }
    }
    if highest_floor != TraitFloor::default() {
        Ok(Some(highest_floor))
    } else {
        // in case nothing has a floor try again with all traits without filtering
        token_traits_filtered = token_traits.clone();
        let mut highest_floor = TraitFloor::default();
        for t in token_traits_filtered {
            let trait_listings = get_trait_listings(conn, collection_slug, &t.trait_id).await?;
            if !trait_listings.is_empty() && trait_listings[0].price > highest_floor.floor_price {
                highest_floor = TraitFloor {
                    trait_id: t.trait_id.clone(),
                    token_id: trait_listings[0].token_id,
                    floor_price: trait_listings[0].price,
                }
            }
        }

        if highest_floor != TraitFloor::default() {
            Ok(Some(highest_floor))
        } else {
            Ok(None)
        }
    }
}

pub async fn get_flattening_staircase_price(
    conn: &mut PgConnection,
    collection_slug: &str,
    collection_floor: f64,
    token_traits: Vec<TraitRarities>,
    cutoff: f64,
) -> Result<Option<f64>> {
    let floors = get_all_traits_floor(conn, collection_slug, token_traits, cutoff)
        .await?
        .iter()
        .map(|f| f.floor_price)
        .collect::<Vec<_>>();

    if floors.is_empty() {
        return Ok(None);
    }
    let mut price = floors[0];

    for (i, f) in floors.iter().enumerate().skip(1) {
        price += (f - collection_floor) / (2f64 + (f - floors[i - 1]))
    }

    Ok(Some(price))
}

pub async fn get_all_traits_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_traits: Vec<TraitRarities>,
    cutoff: f64,
) -> Result<Vec<TraitFloor>> {
    let mut token_traits_filtered = token_traits
        .iter()
        .filter(|t| t.rarity < cutoff)
        .cloned()
        .collect::<Vec<_>>();

    // if all traits are above the average rarity
    if token_traits_filtered.is_empty() {
        token_traits_filtered = token_traits.clone();
    }

    let mut floors = vec![];
    for t in token_traits_filtered {
        let trait_listings = get_trait_listings(conn, collection_slug, &t.trait_id).await?;
        if !trait_listings.is_empty() {
            floors.push(TraitFloor {
                trait_id: t.trait_id.clone(),
                token_id: trait_listings[0].token_id,
                floor_price: trait_listings[0].price,
            })
        }
    }

    floors.sort_by(|a, b| b.floor_price.partial_cmp(&a.floor_price).unwrap());

    Ok(floors)
}

pub async fn get_rarest_trait_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_traits: Vec<TraitRarities>,
) -> Result<Option<RarestTraitFloor>> {
    let mut token_traits = token_traits.clone();
    token_traits.sort_by(|a, b| a.rarity.partial_cmp(&b.rarity).unwrap());

    if token_traits.is_empty() {
        return Ok(None);
    }

    let listings = get_trait_listings(conn, collection_slug, &token_traits[0].trait_id).await?;
    if !listings.is_empty() {
        Ok(Some(RarestTraitFloor {
            trait_id: token_traits[0].trait_id.clone(),
            token_id: listings[0].token_id,
            floor_price: listings[0].price,
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_rarity_weighted_floor(
    conn: &mut PgConnection,
    collection_slug: &str,
    traits: Vec<TraitRarities>,
    cutoff: f64,
) -> Result<Option<f64>> {
    let token_traits = traits
        .clone()
        .into_iter()
        .filter(|t| t.rarity < (cutoff / 3f64))
        .collect::<Vec<_>>();

    if token_traits.is_empty() {
        return Ok(get_rarest_trait_floor(conn, collection_slug, traits)
            .await?
            .map(|f| f.floor_price));
    }

    let mut floors = vec![];
    for t in token_traits {
        let trait_listings = get_trait_listings(conn, collection_slug, &t.trait_id).await?;
        if !trait_listings.is_empty() {
            floors.push((
                trait_listings[0].token_id,
                trait_listings[0].price,
                t.rarity,
            ));
        }
    }

    if floors.is_empty() {
        return Ok(None);
    }

    floors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut price = floors[0].1;
    for (idx, f) in floors.iter().enumerate().skip(1) {
        price += f.1 / (2f64 * f.2 / floors[idx - 1].2)
    }
    Ok(Some(price))
}

pub async fn get_last_sale_price(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<Option<f64>> {
    let asset_sales = get_asset_sales(conn, collection_slug, token_id).await?;

    if !asset_sales.is_empty() {
        Ok(Some(asset_sales.last().unwrap().price))
    } else {
        Ok(None)
    }
}

pub async fn get_most_valued_trait_last_sale_avg(
    conn: &mut PgConnection,
    collection_slug: &str,
    nr: Option<usize>,
    token_traits: Vec<TraitRarities>,
    cutoff: f64,
) -> Result<Option<f64>> {
    let most_valuable_trait =
        get_most_valued_trait_floor(conn, collection_slug, token_traits, cutoff).await?;

    let trait_sales = if most_valuable_trait.is_some() {
        get_average_trait_sales_nr(
            conn,
            collection_slug,
            &most_valuable_trait.unwrap().trait_id,
            nr,
        )
        .await?
    } else {
        None
    };

    Ok(trait_sales)
}

pub async fn get_last_sale_relative_to_collection_avg(
    conn: &mut PgConnection,
    collection_slug: &str,
    last_sale: &Option<TokenSale>,
) -> Result<Option<f64>> {
    if last_sale.is_some() {
        let last_sale = last_sale.clone().unwrap();
        let avg_at_sale =
            match get_average_collection_sales_at_ts(conn, collection_slug, &last_sale.time).await?
            {
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

        Ok(Some((last_sale.price / avg_at_sale) * avg_now))
    } else {
        Ok(None)
    }
}

pub async fn get_last_sale_relative_to_trait_avg(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
    last_sale: &Option<TokenSale>,
) -> Result<Option<f64>> {
    if last_sale.is_some() {
        let last_sale = last_sale.clone().unwrap();
        let avg_at_sale =
            match get_average_trait_sales_at_ts(conn, collection_slug, trait_name, &last_sale.time)
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

        Ok(Some((last_sale.price / avg_at_sale) * avg_now))
    } else {
        Ok(None)
    }
}
