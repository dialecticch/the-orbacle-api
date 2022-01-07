use crate::analyzers::{prices::get_most_valued_trait_floor, rarities::get_trait_rarities};
use crate::opensea::{types::AssetsRequest, OpenseaAPIClient};
use crate::profiles::token::price_profile::PriceProfile;
use crate::storage::read::read_collection;
use anyhow::Result;
use sqlx::PgConnection;
use std::collections::HashMap;

pub async fn get_value_for_wallet(
    conn: &mut PgConnection,
    collection_slug: &str,
    wallet: &str,
) -> Result<(f64, HashMap<String, PriceProfile>)> {
    let client = OpenseaAPIClient::new(2);

    let collection = read_collection(conn, collection_slug).await?;

    let req = AssetsRequest::new()
        .asset_contract_address(&collection.address)
        .owner(wallet)
        .build();

    let assets = client.get_assets(req).await?;

    let ids = assets.into_iter().map(|a| a.token_id).collect::<Vec<_>>();

    let mut value = 0f64;
    let mut map = HashMap::<String, PriceProfile>::new();
    for token_id in ids {
        let token_traits = get_trait_rarities(conn, &collection_slug, token_id).await?;

        let rarest_trait = token_traits[0].trait_id.clone();

        let most_valuable_trait = get_most_valued_trait_floor(
            conn,
            &collection_slug,
            token_traits.clone(),
            collection.rarity_cutoff,
        )
        .await?;

        let profile = PriceProfile::make(
            conn,
            &collection.slug,
            token_id as i32,
            token_traits,
            &rarest_trait,
            &most_valuable_trait,
            collection.rarity_cutoff,
        )
        .await
        .unwrap();

        value += profile.custom_price.unwrap_or(profile.max_price);

        map.insert(token_id.to_string(), profile);
    }

    Ok((value, map))
}
