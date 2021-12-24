use super::{
    liquidty_profile::LiquidityProfile, price_profile::PriceProfile, rarity_profile::RarityProfile,
};
use crate::analyzers::listings::*;
use crate::storage::read::read_assets_for_owner;
use crate::storage::read::{read_asset, read_collection};
use anyhow::Result;
use sqlx::PgConnection;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema)]
pub struct TokenProfile {
    pub opensea: String,
    pub name: String,
    pub owner: String,
    pub collection_slug: String,
    pub token_id: i32,
    pub image_url: String,
    pub listing_price: Option<f64>,
    pub owner_tokens_in_collection: i64,
    pub price_profile: PriceProfile,
    pub liquidity_profile: LiquidityProfile,
    pub rarity_profile: RarityProfile,
}

impl TokenProfile {
    pub async fn make(
        conn: &mut PgConnection,
        collection_slug: &str,
        token_id: i32,
        cutoff: i32,
    ) -> Result<Self> {
        log::info!("Getting asset");
        let asset = read_asset(conn, collection_slug, token_id).await?;

        log::info!("Getting collection_address");
        let collection_address = read_collection(conn, collection_slug).await?.address;

        log::info!("Getting listing_price");
        let listing_price = get_token_listings(conn, collection_slug, vec![token_id]).await?[0].1;

        Ok(Self {
            opensea: format!(
                "https://opensea.io/assets/{}/{}",
                collection_address, token_id
            ),
            name: asset.name,
            owner: asset.owner.clone(),
            collection_slug: collection_slug.to_string(),
            token_id,
            image_url: asset.image_url,
            listing_price,
            owner_tokens_in_collection: read_assets_for_owner(conn, collection_slug, &asset.owner)
                .await?
                .unwrap_or_default(),
            price_profile: PriceProfile::make(conn, collection_slug, token_id, cutoff).await?,
            liquidity_profile: LiquidityProfile::make(conn, collection_slug, token_id).await?,
            rarity_profile: RarityProfile::make(conn, collection_slug, token_id).await?,
        })
    }
}
