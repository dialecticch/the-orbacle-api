use anyhow::Result;
use local::profiles::token_profile::TokenProfile;
use local::storage::establish_connection;
static COLLECTION: &str = "forgottenruneswizardscult";
#[tokio::main]
pub async fn main() -> Result<()> {
    let pool = establish_connection().await;
    let mut conn = pool.acquire().await?;

    let items = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut value = 0f64;
    for item in items {
        println!("Making: {:?}", item);
        let profile = TokenProfile::make(&mut conn, COLLECTION, item, 1).await?;
        value += profile.price_profile.max_price;

        println!("{}", serde_json::to_string_pretty(&profile).unwrap());
    }

    println!("Total Value {:?}", value);

    Ok(())
}
