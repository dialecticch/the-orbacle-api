use super::{
    collection_profile::CollectionProfile, liquidty_profile::LiquidityProfile,
    price_profile::PriceProfile, rarity_profile::RarityProfile,
};
use crate::analyzers::listings::*;
use crate::analyzers::prices::get_most_valued_trait_floor;
use crate::analyzers::rarities::get_trait_rarities;
use crate::from_wei;
use crate::storage::read::{read_asset, read_assets_for_owner, read_listings_token_after_ts};
use crate::storage::Collection;
use anyhow::Result;
use chrono::{Duration, Utc};
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
    pub nr_listings_30d: i32,
    pub owner_tokens_in_collection: i32,
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
        let listing_price = if let Some(t) =
            get_token_listings(conn, &collection_slug, vec![token_id])
                .await?
                .first()
        {
            t.price
        } else {
            None
        };

        let nr_listings_30d = read_listings_token_after_ts(
            conn,
            &collection_slug,
            token_id,
            &(Utc::now() - Duration::days(30)).naive_utc(),
        )
        .await?
        .len() as i32;

        let token_traits = get_trait_rarities(conn, &collection_slug, token_id).await?;

        let rarest_trait = if let Some(t) = token_traits.first() {
            t.trait_id.clone()
        } else {
            String::default()
        };

        let most_valuable_trait = get_most_valued_trait_floor(
            conn,
            &collection_slug,
            token_traits.clone(),
            collection.rarity_cutoff,
        )
        .await?;

        let price_profile = PriceProfile::make(
            conn,
            &collection_slug,
            token_id,
            token_traits.clone(),
            &rarest_trait,
            &most_valuable_trait,
            collection.rarity_cutoff,
        )
        .await?;

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
            nr_listings_30d,
            owner_tokens_in_collection: read_assets_for_owner(conn, &collection_slug, &asset.owner)
                .await?
                .unwrap_or_default() as i32,
            liquidity_profile: LiquidityProfile::make(
                conn,
                &collection_slug,
                token_id,
                &rarest_trait,
                price_profile.max_price,
                &most_valuable_trait.clone().map(|t| t.trait_id),
            )
            .await?,
            price_profile,
            collection_profile: CollectionProfile::make(conn, &collection_slug).await?,
            rarity_profile: RarityProfile::make(
                conn,
                &collection_slug,
                token_id,
                &rarest_trait,
                &most_valuable_trait.clone().map(|t| t.trait_id),
            )
            .await?,
        })
    }
}
