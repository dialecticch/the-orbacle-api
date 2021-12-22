use anyhow::Result;
use local::profiles::price_profile::PriceProfile;

static COLLECTION: &str = "forgottenruneswizardscult";
static TOKEN_ID: i32 = 4896;

#[tokio::main]
pub async fn main() -> Result<()> {
    let profile = PriceProfile::make(COLLECTION, 1587).await?;

    println!("{}", serde_json::to_string_pretty(&profile).unwrap());

    let profile = PriceProfile::make(COLLECTION, 7690).await?;

    println!("{}", serde_json::to_string_pretty(&profile).unwrap());

    let profile = PriceProfile::make(COLLECTION, 6697).await?;

    println!("{}", serde_json::to_string_pretty(&profile).unwrap());

    let profile = PriceProfile::make(COLLECTION, 3801).await?;

    println!("{}", serde_json::to_string_pretty(&profile).unwrap());

    let profile = PriceProfile::make(COLLECTION, 2433).await?;

    println!("{}", serde_json::to_string_pretty(&profile).unwrap());

    let profile = PriceProfile::make(COLLECTION, 2609).await?;

    println!("{}", serde_json::to_string_pretty(&profile).unwrap());

    Ok(())
}
