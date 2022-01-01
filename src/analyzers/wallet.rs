use crate::analyzers::rarities::get_trait_rarities;
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
        let token_traits = get_trait_rarities(conn, &collection.slug, token_id as i32)
            .await
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

        let profile = PriceProfile::make(
            conn,
            &collection.slug,
            token_id as i32,
            token_traits,
            collection.rarity_cutoff,
        )
        .await
        .unwrap();

        value += profile.max_price;

        map.insert(token_id.to_string(), profile);
    }

    Ok((value, map))
}
