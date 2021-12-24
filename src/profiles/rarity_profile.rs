use crate::analyzers::prices::*;
use crate::storage::read::read_asset;
use anyhow::Result;
use sqlx::PgConnection;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema)]
pub struct RarityProfile {
    pub rarest_trait: String,
    pub most_valued_trait: Option<String>,
    pub rarity_score: f64,
    pub unique_traits: i32,
    pub unique_3_trait_combinations: i32,
    pub unique_4_trait_combinations: i32,
    pub unique_5_trait_combinations: i32,
}
impl RarityProfile {
    pub async fn make(
        conn: &mut PgConnection,
        collection_slug: &str,
        token_id: i32,
    ) -> Result<Self> {
        log::info!("Getting asset");
        let asset = read_asset(conn, collection_slug, token_id).await?;

        log::info!("Getting rarest_trait");
        let rarest_trait = get_rarest_trait_floor(conn, collection_slug, token_id)
            .await?
            .0;

        log::info!("Getting most_valuable_trait_resp");
        let most_valuable_trait_resp =
            get_most_valued_trait_floor(conn, collection_slug, token_id, 0.03).await?;

        log::info!("Getting most_valued_trait_floor");
        let most_valued_trait = most_valuable_trait_resp.0;

        Ok(Self {
            rarest_trait,
            most_valued_trait,
            rarity_score: asset.rarity_score,
            unique_traits: asset.unique_traits,
            unique_3_trait_combinations: asset.unique_3_trait_combinations,
            unique_4_trait_combinations: asset.unique_4_trait_combinations,
            unique_5_trait_combinations: asset.unique_5_trait_combinations,
        })
    }
}
