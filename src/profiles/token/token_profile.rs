use super::{
    collection_profile::CollectionProfile, liquidty_profile::LiquidityProfile,
    price_profile::PriceProfile, rarity_profile::RarityProfile,
};
use crate::analyzers::listings::*;
use crate::analyzers::rarities::get_trait_rarities;
use crate::from_wei;
use crate::storage::read::read_asset;
use crate::storage::read::read_assets_for_owner;
use crate::storage::Collection;
use anyhow::Result;
use sqlx::PgConnection;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema, Clone)]
pub struct TokenProfile {
    pub opensea: String,
    pub name: String,
    pub owner: String,
    pub collection_slug: String,
    pub collection_name: String,
    pub token_id: i32,
    pub image_url: String,
    pub listing_price: Option<f64>,
    pub owner_tokens_in_collection: i64,
    pub collection_profile: CollectionProfile,
    pub price_profile: PriceProfile,
    pub liquidity_profile: LiquidityProfile,
    pub rarity_profile: RarityProfile,
}

impl TokenProfile {
    pub async fn make(
        conn: &mut PgConnection,
        collection: Collection,
        token_id: i32,
    ) -> Result<Self> {
        log::info!("Getting asset");

        let collection_slug = collection.slug;

        let asset = read_asset(conn, &collection_slug, token_id).await?;

        log::info!("Getting collection_address");
        let collection_address = collection.address;

        log::info!("Getting listing_price");
        let listing_price = get_token_listings(conn, &collection_slug, vec![token_id]).await?[0].1;

        let token_traits = get_trait_rarities(conn, &collection_slug, token_id).await?;

        Ok(Self {
            opensea: format!(
                "https://opensea.io/assets/{}/{}",
                collection_address, token_id
            ),
            name: asset.name,
            owner: asset.owner.clone(),
            collection_slug: collection_slug.to_string(),
            collection_name: collection.name.to_string(),
            token_id,
            image_url: asset.image_url,
            listing_price: listing_price.map(from_wei),
            owner_tokens_in_collection: read_assets_for_owner(conn, &collection_slug, &asset.owner)
                .await?
                .unwrap_or_default(),
            price_profile: PriceProfile::make(
                conn,
                &collection_slug,
                token_id,
                token_traits.clone(),
                collection.rarity_cutoff,
            )
            .await?,
            collection_profile: CollectionProfile::make(conn, &collection_slug).await?,
            liquidity_profile: LiquidityProfile::make(
                conn,
                &collection_slug,
                token_id,
                token_traits.clone(),
                collection.rarity_cutoff,
            )
            .await?,
            rarity_profile: RarityProfile::make(
                conn,
                &collection_slug,
                token_id,
                token_traits.clone(),
                collection.rarity_cutoff,
            )
            .await?,
        })
    }
}
