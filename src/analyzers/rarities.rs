use crate::storage::read::*;
use crate::storage::Trait;
use anyhow::Result;
use sqlx::PgConnection;

pub async fn get_trait_rarities(
    conn: &mut PgConnection,
    collection_slug: &str,
    token_id: i32,
) -> Result<Vec<(String, f64)>> {
    let collection = read_collection(conn, collection_slug).await?;
    let traits = read_traits_for_asset(conn, collection_slug, token_id).await?;
    let mut traits_vec: Vec<(String, i32)> = traits.into_iter().collect();
    traits_vec.sort_by(|a, b| a.1.cmp(&b.1));

    Ok(traits_vec
        .into_iter()
        .map(|(t, c)| (t, c as f64 / collection.total_supply as f64))
        .collect())
}

pub fn get_collection_avg_trait_rarity(traits: &[Trait]) -> Result<f64> {
    let traits: Vec<i32> = traits
        .iter()
        .map(|k| k.trait_count as i32)
        .collect::<Vec<_>>();

    Ok(traits.iter().sum::<i32>() as f64 / traits.len() as f64)
}
