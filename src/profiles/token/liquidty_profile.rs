use crate::analyzers::listings::*;
use crate::analyzers::velocity::*;
use crate::storage::read::read_trait;
use anyhow::Result;
use sqlx::PgConnection;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema, Clone)]
pub struct LiquidityProfile {
    pub rarest_trait_nr_listed: (usize, usize),
    pub mvt_nr_listed: (usize, usize),
    pub rarest_trait_sale_count: usize,
    pub mvt_sale_count: usize,
    pub lowest_trait_sales: usize,
    pub avg_sale_count: f64,
}

impl LiquidityProfile {
    pub async fn make(
        conn: &mut PgConnection,
        collection_slug: &str,
        token_id: i32,
        rarest_trait: &str,
        most_valuable_trait: &Option<String>,
    ) -> Result<Self> {
        let rarest_trait_count = read_trait(conn, collection_slug, &rarest_trait)
            .await?
            .trait_count;

        log::info!("Getting rarest_trait_nr_listed");
        let rarest_trait_nr_listed =
            get_trait_nr_listed(conn, collection_slug, &rarest_trait.clone()).await?;

        let (mvt_trait_count, mvt_nr_listed, mvt_sale_count) = match most_valuable_trait.clone() {
            Some(t) => (
                read_trait(conn, collection_slug, &t).await?.trait_count,
                get_trait_nr_listed(conn, collection_slug, &t).await?,
                get_sale_count_trait(conn, collection_slug, &t, 60).await?,
            ),
            None => (0, 0, 0),
        };

        log::info!("Getting avg_sale_count_30d");
        let avg_sale_count = get_avg_sale_count(conn, collection_slug, token_id, 60).await?;

        log::info!("Getting lowest_trait_sales");
        let lowest_trait_sales = get_lowest_sale_count(conn, collection_slug, token_id, 60).await?;

        log::info!("Getting rarest_trait_sale_count");
        let rarest_trait_sale_count =
            get_sale_count_trait(conn, collection_slug, &rarest_trait.clone(), 60).await?;

        Ok(Self {
            rarest_trait_nr_listed: (rarest_trait_nr_listed, rarest_trait_count as usize),
            mvt_nr_listed: (mvt_nr_listed, mvt_trait_count as usize),
            lowest_trait_sales: lowest_trait_sales.1,
            rarest_trait_sale_count,
            avg_sale_count,
            mvt_sale_count,
        })
    }
}
