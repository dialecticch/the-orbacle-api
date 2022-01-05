use crate::analyzers::prices::*;
use crate::analyzers::sales::*;
use crate::custom::read_custom_price;
use anyhow::Result;
use sqlx::PgConnection;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema)]
pub struct PriceProfile {
    pub collection_floor: f64,
    pub last_sale: Option<f64>,
    pub most_rare_trait_floor: Option<f64>,
    pub most_valued_trait_floor: Option<f64>,
    pub rarity_weighted_floor: f64,
    pub avg_last_three_mvt_sales: Option<f64>,
    pub last_sale_relative_collection_avg: Option<f64>,
    pub last_sale_relative_mvt_avg: Option<f64>,
    pub custom_price: Option<f64>,
    pub max_price: f64,
}

impl PriceProfile {
    pub async fn make(
        conn: &mut PgConnection,
        collection_slug: &str,
        token_id: i32,
        token_traits: Vec<(String, f64)>,
        cutoff: f64,
    ) -> Result<Self> {
        log::info!("Getting collection_floor");
        let collection_floor = get_collection_floor(conn, collection_slug).await?;

        log::info!("Getting most_rare_trait_floor");
        let most_rare_trait_floor =
            get_rarest_trait_floor(conn, collection_slug, token_traits.clone())
                .await?
                .2;

        log::info!("Getting most_valuable_trait_resp");
        let most_valuable_trait_resp =
            get_most_valued_trait_floor(conn, collection_slug, token_traits.clone(), cutoff)
                .await?;

        log::info!("Getting most_valued_trait_floor");
        let most_valued_trait = most_valuable_trait_resp.0;
        let most_valued_trait_floor = most_valuable_trait_resp.1;

        log::info!("Getting rarity_weighted_floor");
        let rarity_weighted_floor =
            get_rarity_weighted_floor(conn, collection_slug, token_traits, cutoff).await?;

        log::info!("Getting last_sale");
        let last_sale = get_last_sale_price(conn, collection_slug, token_id).await?;

        log::info!("Getting avg_last_three_mvt_sales");
        let avg_last_three_mvt_sales = get_average_trait_sales_nr(
            conn,
            collection_slug,
            &most_valued_trait.clone().unwrap_or_default(),
            Some(3),
        )
        .await?;

        log::info!("Getting last_sale_relative_collection_avg");
        let last_sale_relative_collection_avg =
            get_last_sale_relative_to_collection_avg(conn, collection_slug, token_id).await?;

        log::info!("Getting last_sale_relative_mvt_avg");
        let last_sale_relative_mvt_avg = get_last_sale_relative_to_trait_avg(
            conn,
            collection_slug,
            &most_valued_trait.clone().unwrap_or_default(),
            token_id,
        )
        .await?;

        let mut prices = vec![
            collection_floor,
            last_sale.unwrap_or(0f64),
            most_rare_trait_floor.unwrap_or(0f64),
            most_valued_trait_floor.unwrap_or(0f64),
            rarity_weighted_floor,
            avg_last_three_mvt_sales.unwrap_or(0f64),
            last_sale_relative_collection_avg.unwrap_or(0f64),
            last_sale_relative_mvt_avg.unwrap_or(0f64),
        ];

        prices.sort_by(|a, b| b.partial_cmp(a).unwrap());

        Ok(Self {
            collection_floor,
            last_sale,
            most_rare_trait_floor,
            most_valued_trait_floor,
            rarity_weighted_floor,
            avg_last_three_mvt_sales,
            last_sale_relative_collection_avg,
            last_sale_relative_mvt_avg,
            custom_price: read_custom_price(collection_slug, token_id)?,
            max_price: prices[0],
        })
    }
}
