use crate::analyzers::prices::*;
use crate::storage::read::read_asset;
use anyhow::Result;
use sqlx::PgConnection;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema, Clone)]
pub struct RarityProfile {
    pub rarest_trait: String,
    pub most_valued_trait: Option<String>,
    pub unique_traits: i32,
    pub traits_3_combination_overlap: i32,
    pub traits_3_combination_overlap_ids: Vec<i32>,
    pub traits_4_combination_overlap: i32,
    pub traits_4_combination_overlap_ids: Vec<i32>,
    pub traits_5_combination_overlap: i32,
    pub traits_5_combination_overlap_ids: Vec<i32>,
}
impl RarityProfile {
    pub async fn make(
        conn: &mut PgConnection,
        collection_slug: &str,
        token_id: i32,
        token_traits: Vec<(String, f64)>,
        cutoff: f64,
    ) -> Result<Self> {
        log::info!("Getting asset");
        let asset = read_asset(conn, collection_slug, token_id).await?;

        log::info!("Getting rarest_trait");
        let rarest_trait = get_rarest_trait_floor(conn, collection_slug, token_traits.clone())
            .await?
            .0;

        log::info!("Getting most_valuable_trait_resp");
        let most_valuable_trait_resp =
            get_most_valued_trait_floor(conn, collection_slug, token_traits, cutoff).await?;

        log::info!("Getting most_valued_trait_floor");
        let most_valued_trait = most_valuable_trait_resp.0;

        Ok(Self {
            rarest_trait,
            most_valued_trait,
            unique_traits: asset.unique_traits,
            traits_3_combination_overlap: asset.traits_3_combination_overlap,
            traits_4_combination_overlap: asset.traits_4_combination_overlap,
            traits_5_combination_overlap: asset.traits_5_combination_overlap,
            traits_3_combination_overlap_ids: asset.traits_3_combination_overlap_ids,
            traits_4_combination_overlap_ids: asset.traits_4_combination_overlap_ids,
            traits_5_combination_overlap_ids: asset.traits_5_combination_overlap_ids,
        })
    }
}
