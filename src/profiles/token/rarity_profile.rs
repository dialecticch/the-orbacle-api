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
        rarest_trait: &str,
        most_valued_trait: &Option<String>,
    ) -> Result<Self> {
        log::info!("Getting asset");
        let asset = read_asset(conn, collection_slug, token_id).await?;

        Ok(Self {
            rarest_trait: rarest_trait.into(),
            most_valued_trait: most_valued_trait.clone(),
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
