use super::price_profile::PriceProfile;
use crate::analyzers::wallet::get_value_for_wallet;
use anyhow::Result;
use sqlx::PgConnection;
use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema, Clone)]
pub struct WalletProfile {
    pub total_value: f64,
    pub tokens: HashMap<String, PriceProfile>,
}

impl WalletProfile {
    pub async fn make(
        conn: &mut PgConnection,
        collection_slug: &str,
        wallet: &str,
    ) -> Result<Self> {
        let (value, profiles) = get_value_for_wallet(conn, collection_slug, wallet).await?;

        Ok(Self {
            total_value: value,
            tokens: profiles,
        })
    }
}
