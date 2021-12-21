use anyhow::Result;
use local::opensea::OpenseaAPIClient;
use local::storage::establish_connection;
use local::storage::read::*;

static COLLECTION: &str = "forgottenruneswizardscult";
static TRAIT: &str = "dream master";

#[tokio::main]
pub async fn main() -> Result<()> {
    let pool = establish_connection().await;
    let mut conn = pool.acquire().await?;

    let c = read_collection(&mut conn, COLLECTION).await?;
    let assets_with_trait = read_assets_with_trait(&mut conn, COLLECTION, TRAIT)
        .await
        .unwrap();

    let ids: Vec<_> = assets_with_trait
        .into_iter()
        .map(|a| a.token_id as u64)
        .collect();

    println!("Fetching price for: {:?} tokens", ids.len());

    let client = OpenseaAPIClient::new();

    let mut all_assets: Vec<_> = client
        .fetch_token_ids(COLLECTION, ids)
        .await?
        .into_iter()
        .filter(|a| a.sell_orders.is_some())
        .collect::<Vec<_>>();

    all_assets.sort_by(|a, b| {
        a.sell_orders.clone().unwrap()[0]
            .current_price
            .partial_cmp(&b.sell_orders.clone().unwrap()[0].current_price)
            .unwrap()
    });

    for a in all_assets.clone() {
        if a.sell_orders.is_some() {
            println!(
                "{} ETH - {:?}",
                a.sell_orders.unwrap()[0].current_price / 10f64.powf(18f64),
                format!("https://opensea.io/assets/{}/{}", c.address, a.token_id)
            );
        }
    }

    Ok(())
}
