use crate::analyzers::prices::*;
use crate::analyzers::sales::*;
use crate::analyzers::*;
use crate::custom::read_custom_price;
use anyhow::Result;
use sqlx::PgConnection;

#[derive(Debug, serde::Serialize, serde::Deserialize, rweb::Schema, Clone)]
pub struct PriceProfile {
    pub collection_floor: f64,
    pub last_sale: Option<f64>,
    pub most_rare_trait_floor: Option<f64>,
    pub most_valued_trait_floor: Option<f64>,
    pub rarity_weighted_floor: Option<f64>,
    pub avg_last_three_mvt_sales: Option<f64>,
    pub last_sale_relative_collection_avg: Option<f64>,
    pub last_sale_relative_mvt_avg: Option<f64>,
    pub custom_price: Option<f64>,
    pub max_price: f64,
    pub min_price: f64,
    pub avg_price: f64,
}

impl PriceProfile {
    pub async fn make(
        conn: &mut PgConnection,
        collection_slug: &str,
        token_id: i32,
        token_traits: Vec<TraitRarities>,
        rarest_trait: &str,
        most_valuable_trait: &Option<TraitFloor>,
        cutoff: f64,
    ) -> Result<Self> {
        log::info!("Getting collection_floor");
        let collection_floor = get_collection_floor(conn, collection_slug).await?;

        log::info!("Getting most_rare_trait_floor");
        let most_rare_trait_floor =
            get_trait_floor(conn, collection_slug, <&str>::clone(&rarest_trait))
                .await?
                .map(|t| t.floor_price);

        log::info!("Getting last_sale");
        let last_sale = get_asset_sales(conn, collection_slug, token_id)
            .await?
            .last()
            .cloned();

        log::info!("Getting mvt data");
        let (most_valued_trait_floor, avg_last_three_mvt_sales, last_sale_relative_mvt_avg) =
            match most_valuable_trait {
                Some(t) => (
                    Some(t.floor_price),
                    get_average_trait_sales_nr(conn, collection_slug, &t.trait_id, Some(3)).await?,
                    get_last_sale_relative_to_trait_avg(
                        conn,
                        collection_slug,
                        &t.trait_id,
                        &last_sale,
                    )
                    .await?,
                ),
                None => (None, None, None),
            };

        log::info!("Getting rarity_weighted_floor");
        let rarity_weighted_floor =
            get_rarity_weighted_floor(conn, collection_slug, token_traits, cutoff).await?;

        log::info!("Getting last_sale_relative_collection_avg");
        let last_sale_relative_collection_avg =
            get_last_sale_relative_to_collection_avg(conn, collection_slug, &last_sale).await?;

        let mut prices = vec![
            collection_floor,
            last_sale.clone().unwrap_or_default().price,
            most_rare_trait_floor.unwrap_or(0f64),
            most_valued_trait_floor.unwrap_or(0f64),
            rarity_weighted_floor.unwrap_or(0f64),
            avg_last_three_mvt_sales.unwrap_or(0f64),
            last_sale_relative_collection_avg.unwrap_or(0f64),
            last_sale_relative_mvt_avg.unwrap_or(0f64),
        ]
        .into_iter()
        .filter(|p| p > &0f64)
        .collect::<Vec<_>>();

        prices.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let max_price = prices[0];

        prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min_price = f64::max(prices[0], collection_floor);

        let avg_price = (max_price + min_price) / 2f64;

        Ok(Self {
            collection_floor,
            last_sale: last_sale.map(|l| l.price),
            most_rare_trait_floor,
            most_valued_trait_floor,
            rarity_weighted_floor,
            avg_last_three_mvt_sales,
            last_sale_relative_collection_avg,
            last_sale_relative_mvt_avg,
            custom_price: read_custom_price(collection_slug, token_id)?,
            max_price,
            min_price,
            avg_price,
        })
    }
}
