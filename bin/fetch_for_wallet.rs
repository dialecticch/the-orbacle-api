use anyhow::Result;
use local::analyzers::rarities::get_trait_rarities;
use local::opensea::{types::AssetsRequest, OpenseaAPIClient};
use local::profiles::token::price_profile::PriceProfile;
use local::storage::{establish_connection, read::read_collection};

static COLLECTION: &str = "forgottenruneswizardscult";

#[tokio::main]
pub async fn main() -> Result<()> {
    let pool = establish_connection().await;
    let mut conn = pool.acquire().await?;

    let client = OpenseaAPIClient::new(2);

    let collection = read_collection(&mut conn, COLLECTION).await?;

    let req = AssetsRequest::new()
        .asset_contract_address(&collection.address)
        .owner("address")
        .build();

    let assets = client.get_assets(req).await?;

    let ids = assets.into_iter().map(|a| a.token_id).collect::<Vec<_>>();

    let mut value = 0f64;
    for token_id in ids {
        let token_traits = get_trait_rarities(&mut conn, &collection.slug, token_id as i32)
            .await
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();

        let profile = PriceProfile::make(
            &mut conn,
            &collection.slug,
            token_id as i32,
            token_traits,
            collection.rarity_cutoff,
        )
        .await
        .unwrap();

        value += profile.max_price;

        println!("{} - {}", token_id, profile.max_price);
    }

    println!("Total Value {:?}", value);

    Ok(())
}
