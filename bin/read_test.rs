use anyhow::Result;
use local::storage::establish_connection;
use local::storage::read::*;

static COLLECTION: &'static str = "forgottenruneswizardscult";
static TRAIT: &'static str = "illuminatus";

#[tokio::main]
pub async fn main() -> Result<()> {
    let pool = establish_connection().await;
    let mut conn = pool.acquire().await?;

    let c = get_collection(&mut conn, COLLECTION).await?;

    println!("Collection: \n{:?}\n", c);

    let t = get_trait(&mut conn, COLLECTION, TRAIT).await?;

    println!("Trait: \n{:?}\n", t);

    let assets_with_trait = get_assets_with_trait(&mut conn, COLLECTION, TRAIT)
        .await
        .unwrap();

    println!("Assets with Trait: \n{:?}\n", &assets_with_trait.len());

    let at = get_traits_for_asset(&mut conn, COLLECTION, 777).await?;

    println!("Traits for Asset: \n{:?}\n", at);

    Ok(())
}
