use crate::storage::read::{
    read_collection, read_listing_update_type_count_after_ts, read_nr_listed_for_collection_at_ts,
};
use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgConnection;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema, Default, Clone)]
pub struct CollectionProfile {
    pub banner_image_url: String,
    pub daily_volume: f64,
    pub daily_sales: f64,
    pub daily_avg_price: f64,
    pub weekly_avg_price: f64,
    pub monthly_avg_price: f64,
    pub nr_owners: f64,
    pub avg_trait_rarity: f64,
    pub nr_listed_now: i64,
    pub nr_new_listings_14d: i64,
    pub nr_cancelled_listings_14d: i64,
    pub nr_sales_14d: i64,
}

impl CollectionProfile {
    pub async fn make(conn: &mut PgConnection, collection_slug: &str) -> Result<Self> {
        log::info!("Getting collection");

        let collection = read_collection(conn, collection_slug).await?;

        log::info!("Getting nr_listed_now");
        let nr_listed_now =
            read_nr_listed_for_collection_at_ts(conn, collection_slug, &Utc::now().naive_utc())
                .await?;

        let ts_14d_ago = (Utc::now() - Duration::days(14)).naive_utc();

        Ok(Self {
            banner_image_url: collection.banner_image_url.clone(),
            daily_volume: collection.daily_volume,
            daily_sales: collection.daily_sales,
            daily_avg_price: collection.daily_avg_price,
            weekly_avg_price: collection.weekly_avg_price,
            monthly_avg_price: collection.monthly_avg_price,
            nr_owners: collection.nr_owners,
            avg_trait_rarity: collection.avg_trait_rarity,
            nr_listed_now: nr_listed_now.unwrap_or_default(),
            nr_new_listings_14d: read_listing_update_type_count_after_ts(
                conn,
                collection_slug,
                "created",
                &ts_14d_ago,
            )
            .await?
            .unwrap_or_default(),
            nr_cancelled_listings_14d: read_listing_update_type_count_after_ts(
                conn,
                collection_slug,
                "cancelled",
                &ts_14d_ago,
            )
            .await?
            .unwrap_or_default(),
            nr_sales_14d: read_listing_update_type_count_after_ts(
                conn,
                collection_slug,
                "successful",
                &ts_14d_ago,
            )
            .await?
            .unwrap_or_default(),
        })
    }
}
