use super::price_profile::PriceProfile;
use crate::analyzers::wallet::get_value_for_wallet;
use crate::storage::read::read_asset;
use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema, Clone)]
pub struct TokensInner {
    pub img: String,
    pub opensea: String,
    pub price_profile: PriceProfile,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema, Clone)]
pub struct WalletProfile {
    pub total_value_max: f64,
    pub total_value_min: f64,
    pub total_value_avg: f64,
    pub tokens: HashMap<String, TokensInner>,
}

impl WalletProfile {
    pub async fn make(pool: PgPool, collection_slug: &str, wallet: &str) -> Result<Self> {
        let mut conn = pool.acquire().await?;
        let (value_max, value_min, value_avg, address, profiles) =
            get_value_for_wallet(pool, collection_slug, wallet).await?;

        let mut tokens = HashMap::<String, TokensInner>::new();
        for (t, p) in profiles {
            let asset = read_asset(&mut conn, collection_slug, t.parse::<i32>().unwrap()).await?;
            tokens.insert(
                t.clone(),
                TokensInner {
                    img: asset.image_url,
                    opensea: format!("https://opensea.io/assets/{}/{}", address, t),
                    price_profile: p,
                },
            );
        }

        Ok(Self {
            total_value_max: value_max,
            total_value_min: value_min,
            total_value_avg: value_avg,
            tokens,
        })
    }
}
