use anyhow::Result;
use local::analyzers::{prices::*, rarities::*};
use local::storage::establish_connection;
use local::storage::read::*;

static COLLECTION: &str = "forgottenruneswizardscult";
static TRAIT: &str = "flaming skull";
static TOKEN_ID: i32 = 8046;

#[tokio::main]
pub async fn main() -> Result<()> {
    let pool = establish_connection().await;
    let mut conn = pool.acquire().await?;

    let c = read_collection(&mut conn, COLLECTION).await?;

    println!("Collection: \n{:?}\n", c);

    let t = read_trait(&mut conn, COLLECTION, TRAIT).await?;

    println!("Trait: \n{:?}\n", t);

    let assets_with_trait = read_assets_with_trait(&mut conn, COLLECTION, TRAIT)
        .await
        .unwrap();

    println!("Assets with Trait: \n{:?}\n", &assets_with_trait.len());

    let at = read_traits_for_asset(&mut conn, COLLECTION, TOKEN_ID).await?;

    println!("Traits for Asset: \n{:?}\n", at);

    let at = get_trait_rarities(&mut conn, COLLECTION, TOKEN_ID).await?;

    println!("Rarities for Asset: \n{:?}\n", at);

    let at = get_trait_listing(&mut conn, COLLECTION, &at[0].0).await?;

    println!("Listings of rarest Trait: \n{:?}\n", at);

    let at = get_trait_listing(&mut conn, COLLECTION, TRAIT).await?;

    println!("Listings of {} Trait: \n{:?}\n", TRAIT, at);

    let at = get_trait_floor(&mut conn, COLLECTION, TRAIT).await?;

    println!("Floor of {} Trait: \n{:?}\n", TRAIT, at);

    let at = get_rarest_trait_floor(&mut conn, COLLECTION, TOKEN_ID).await?;

    println!("Floor of rarest Trait: \n{:?}\n", at);

    let at = get_most_valued_trait_floor(&mut conn, COLLECTION, TOKEN_ID, 0.01).await?;

    println!("Floor of most valuable Traits: \n{:?}\n", at);

    let at = get_rarity_weighted_floor(&mut conn, COLLECTION, TOKEN_ID, 0.01).await?;

    println!("Rarity weighted price for {}: \n{:?}\n", TOKEN_ID, at);

    Ok(())
}
