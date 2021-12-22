use anyhow::Result;
use local::profiles::price_profile::TokenProfile;

static COLLECTION: &str = "forgottenruneswizardscult";
#[tokio::main]
pub async fn main() -> Result<()> {
    let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut value = 0f64;
    for item in items {
        println!("Making: {:?}", item);
        let profile = TokenProfile::make(COLLECTION, item, 1).await?;
        value += profile.price_profile.max_price;

        println!("{}", serde_json::to_string_pretty(&profile).unwrap());
    }

    println!("Total Value {:?}", value);

    Ok(())
}
