use super::*;
use crate::from_wei;
use crate::storage::read::*;
use anyhow::Result;
use sqlx::PgConnection;

pub async fn get_token_listings(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_ids: Vec<i32>,
) -> Result<Vec<TokenListing>> {
    let mut map: Vec<TokenListing> = Vec::new();
    for id in token_ids {
        let listing = read_latests_listing_for_asset(conn, collection_slug, id).await?;

        map.push(TokenListing {
            token_id: id,
            price: listing[0].price,
        });
    }

    Ok(map)
}
pub async fn get_trait_listings(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
) -> Result<Vec<TraitListing>> {
    let assets_with_trait = read_assets_with_trait(conn, collection_slug, trait_name)
        .await
        .unwrap();

    let ids: Vec<_> = assets_with_trait.into_iter().map(|a| a.token_id).collect();

    let mut all_assets: Vec<_> = get_token_listings(conn, collection_slug, ids).await?;

    all_assets = all_assets
        .into_iter()
        .filter(|a| a.price.is_some())
        .collect::<Vec<_>>();

    all_assets.sort_by(|a, b| a.price.unwrap().partial_cmp(&b.price.unwrap()).unwrap());

    Ok(all_assets
        .into_iter()
        .map(|t| TraitListing {
            token_id: t.token_id,
            price: from_wei(t.price.unwrap()),
        })
        .collect())
}

pub async fn get_trait_nr_listed(
    conn: &mut PgConnection,
    collection_slug: &str,
    trait_name: &str,
) -> Result<usize> {
    Ok(get_trait_listings(conn, collection_slug, trait_name)
        .await?
        .len())
}
