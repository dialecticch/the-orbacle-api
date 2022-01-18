use crate::analyzers::{prices::get_most_valued_trait_floor, rarities::get_trait_rarities};
use crate::opensea::{types::AssetsRequest, OpenseaAPIClient};
use crate::profiles::token::price_profile::PriceProfile;
use crate::storage::read::read_collection;
use anyhow::Result;
use futures::StreamExt;
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn get_value_for_wallet(
    pool: PgPool,
    collection_slug: &str,
    wallet: &str,
) -> Result<(f64, f64, f64, String, HashMap<String, PriceProfile>)> {
    let client = OpenseaAPIClient::new(2);
    let mut conn = pool.acquire().await?;
    let collection = read_collection(&mut conn, collection_slug).await?;

    let req = AssetsRequest::new()
        .asset_contract_address(&collection.address)
        .owner(wallet)
        .build();

    let assets = client.get_assets(req).await?;

    let ids = assets.into_iter().map(|a| a.token_id).collect::<Vec<_>>();

    let mut value_max = 0f64;
    let mut value_min = 0f64;
    let mut value_avg = 0f64;
    let mut map = HashMap::<String, PriceProfile>::new();
    let mut stream = futures::stream::iter(0..ids.len())
        .map(|i| {
            _get_profile(
                pool.clone(),
                collection_slug,
                ids[i],
                collection.rarity_cutoff,
            )
        })
        .buffer_unordered(6);

    let mut results = vec![];

    while let Some(result) = stream.next().await {
        match result {
            Ok(resp) => {
                if let Some(r) = resp {
                    results.push(r);
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    for profile in results {
        value_max += profile.1.custom_price.unwrap_or(profile.1.max_price);
        value_min += profile.1.custom_price.unwrap_or(profile.1.min_price);
        value_avg += profile.1.custom_price.unwrap_or(profile.1.avg_price);

        map.insert(profile.0.to_string(), profile.1);
    }

    Ok((
        value_max,
        value_min,
        value_avg,
        collection.address.clone(),
        map,
    ))
}

async fn _get_profile(
    pool: PgPool,
    collection_slug: &str,
    token_id: i32,
    cutoff: f64,
) -> Result<Option<(i32, PriceProfile)>> {
    let mut conn = pool.acquire().await?;
    let token_traits = get_trait_rarities(&mut conn, collection_slug, token_id).await?;

    if token_traits.is_empty() {
        return Ok(None);
    }

    println!("{}", token_id);

    let rarest_trait = token_traits[0].trait_id.clone();

    let most_valuable_trait =
        get_most_valued_trait_floor(&mut conn, collection_slug, token_traits.clone(), cutoff)
            .await?;

    let profile = PriceProfile::make(
        &mut conn,
        collection_slug,
        token_id as i32,
        token_traits,
        &rarest_trait,
        &most_valuable_trait,
        cutoff,
    )
    .await
    .unwrap();

    Ok(Some((token_id, profile)))
}
