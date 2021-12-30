use crate::analyzers::listings::*;
use crate::analyzers::prices::*;
use crate::analyzers::velocity::*;
use crate::storage::read::{read_listing_update_type_count_after_ts, read_trait};
use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgConnection;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema)]
pub struct LiquidityProfile {
    pub rarest_trait_nr_listed: (usize, usize),
    pub mvt_nr_listed: (usize, usize),
    pub rarest_trait_sale_frequency_30d: f64,
    pub mvt_sale_frequency_30d: f64,
    pub lowest_sale_frequency_30d: f64,
    pub avg_sale_frequency_30d: f64,
    pub nr_new_listings_14d: i64,
    pub nr_cancelled_listings_14d: i64,
    pub nr_sales_14d: i64,
}

impl LiquidityProfile {
    pub async fn make(
        conn: &mut PgConnection,
        collection_slug: &str,
        token_id: i32,
        token_traits: Vec<(String, f64)>,
        cutoff: f64,
    ) -> Result<Self> {
        log::info!("Getting rarest_trait");
        let rarest_trait = get_rarest_trait_floor(conn, collection_slug, token_traits.clone())
            .await?
            .0;

        let rarest_trait_count = read_trait(conn, collection_slug, &rarest_trait)
            .await?
            .trait_count;

        log::info!("Getting most_valuable_trait_resp");
        let most_valuable_trait_resp =
            get_most_valued_trait_floor(conn, collection_slug, token_traits, cutoff).await?;

        log::info!("Getting most_valued_trait_floor");
        let most_valued_trait = most_valuable_trait_resp.0;

        let mvt_trait_count = read_trait(
            conn,
            collection_slug,
            &most_valued_trait.clone().unwrap_or_default(),
        )
        .await?
        .trait_count;

        log::info!("Getting rarest_trait_nr_listed");
        let rarest_trait_nr_listed =
            get_trait_nr_listed(conn, collection_slug, &rarest_trait.clone()).await?;

        log::info!("Getting mvt_nr_listed");
        let mvt_nr_listed = get_trait_nr_listed(
            conn,
            collection_slug,
            &most_valued_trait.clone().unwrap_or_default(),
        )
        .await?;

        log::info!("Getting avg_sale_frequency_30d");
        let avg_sale_frequency_30d =
            get_avg_sale_frequency(conn, collection_slug, token_id, 30).await?;

        log::info!("Getting lowest_sale_frequency_30d");
        let lowest_sale_frequency_30d =
            get_lowest_sale_frequency(conn, collection_slug, token_id, 30).await?;

        log::info!("Getting mvt_sale_frequency_30d");
        let mvt_sale_frequency_30d = get_sale_frequency_trait(
            conn,
            collection_slug,
            &most_valued_trait.clone().unwrap_or_default(),
            30,
        )
        .await?;

        log::info!("Getting rarest_trait_sale_frequency_30d");
        let rarest_trait_sale_frequency_30d =
            get_sale_frequency_trait(conn, collection_slug, &rarest_trait.clone(), 30).await?;

        let ts_14d_ago = (Utc::now() - Duration::days(14)).naive_utc();

        Ok(Self {
            rarest_trait_nr_listed: (rarest_trait_nr_listed, rarest_trait_count as usize),
            mvt_nr_listed: (mvt_nr_listed, mvt_trait_count as usize),
            lowest_sale_frequency_30d: lowest_sale_frequency_30d.1,
            rarest_trait_sale_frequency_30d,
            avg_sale_frequency_30d,
            mvt_sale_frequency_30d,
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
