use crate::analyzers::liquidty::*;
use crate::analyzers::listings::*;
use crate::storage::read::read_sales_for_collection_above_price_after_ts;
use crate::storage::read::read_trait;
use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgConnection;
#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema, Clone)]
pub struct LiquidityProfile {
    pub rarest_trait_nr_listed: (usize, usize),
    pub mvt_nr_listed: (usize, usize),
    pub rarest_trait_sale_count_60d: usize,
    pub mvt_sale_count_60d: usize,
    pub lowest_trait_sales_60d: usize,
    pub avg_sale_count_60d: f64,
    pub nr_sales_above_max_price_60d: usize,
}

impl LiquidityProfile {
    pub async fn make(
        conn: &mut PgConnection,
        collection_slug: &str,
        token_id: i32,
        rarest_trait: &str,
        max_price: f64,
        most_valuable_trait: &Option<String>,
    ) -> Result<Self> {
        let rarest_trait_count = read_trait(conn, collection_slug, &rarest_trait)
            .await?
            .trait_count;

        log::info!("Getting rarest_trait_nr_listed");
        let rarest_trait_nr_listed =
            get_trait_nr_listed(conn, collection_slug, &rarest_trait.clone()).await?;

        let (mvt_trait_count, mvt_nr_listed, mvt_sale_count_60d) = match most_valuable_trait.clone()
        {
            Some(t) => (
                read_trait(conn, collection_slug, &t).await?.trait_count,
                get_trait_nr_listed(conn, collection_slug, &t).await?,
                get_sale_count_trait(conn, collection_slug, &t, 60).await?,
            ),
            None => (0, 0, 0),
        };

        log::info!("Getting avg_sale_count_30d");
        let avg_sale_count_60d = get_avg_sale_count(conn, collection_slug, token_id, 60).await?;

        log::info!("Getting lowest_trait_sales");
        let lowest_trait_sales_60d =
            get_lowest_sale_count(conn, collection_slug, token_id, 60).await?;

        log::info!("Getting rarest_trait_sale_count");
        let rarest_trait_sale_count_60d =
            get_sale_count_trait(conn, collection_slug, &rarest_trait.clone(), 60).await?;

        log::info!("Getting nr_sales_above_max_price_60d");
        let nr_sales_above_max_price_60d = read_sales_for_collection_above_price_after_ts(
            conn,
            collection_slug,
            max_price,
            &(Utc::now() - Duration::days(60)).naive_utc(),
        )
        .await?
        .len();

        Ok(Self {
            rarest_trait_nr_listed: (rarest_trait_nr_listed, rarest_trait_count as usize),
            mvt_nr_listed: (mvt_nr_listed, mvt_trait_count as usize),
            lowest_trait_sales_60d: lowest_trait_sales_60d.1,
            rarest_trait_sale_count_60d,
            avg_sale_count_60d,
            mvt_sale_count_60d,
            nr_sales_above_max_price_60d,
        })
    }
}
