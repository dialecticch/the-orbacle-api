use anyhow::Result;
use local::profiles::token::token_profile::TokenProfile;
use local::storage::establish_connection;
static COLLECTION: &str = "forgottenruneswizardscult";
use chrono::Utc;
#[tokio::main]
pub async fn main() -> Result<()> {
    let pool = establish_connection().await;
    let mut conn = pool.acquire().await?;

    let mut value = 0f64;
    for item in 0..9433 {
        let b = Utc::now().timestamp_millis();
        let profile = TokenProfile::make(&mut conn, COLLECTION, item).await?;
        let a = Utc::now().timestamp_millis();
        println!("{} ms - {:?} ", a - b, item);
        value += profile.price_profile.max_price;

        //println!("{}", serde_json::to_string_pretty(&profile).unwrap());
    }

    println!("Total Value {:?}", value);

    Ok(())
}
