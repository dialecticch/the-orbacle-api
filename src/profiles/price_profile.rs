use crate::analyzers::prices::*;
use crate::storage::establish_connection;
use anyhow::Result;
#[derive(serde::Serialize, Debug)]
pub struct PriceProfile {
    collection_slug: String,
    token_id: i32,
    listing_price: Option<f64>,
    rarest_trait: String,
    most_valuable_trait: Option<String>,
    collection_floor: f64,
    last_sale: Option<f64>,
    most_rare_trait_floor: Option<f64>,
    most_valued_trait_floor: Option<f64>,
    rarity_weighted_floor: Option<f64>,
    avg_last_three_mvt_sales: Option<f64>,
    last_sale_relative_avg_collection_price: Option<f64>,
    last_sale_relative_avg_mvt_price: Option<f64>,
    max_price: f64,
}

impl PriceProfile {
    pub async fn make(collection_slug: &str, token_id: i32) -> Result<Self> {
        let pool = establish_connection().await;
        let mut conn = pool.acquire().await?;

        let collection_floor = get_collection_floor(&mut conn, collection_slug).await?;

        let listing_price = get_token_listing(&mut conn, collection_slug, token_id).await?;
        let rarest_trait = get_rarest_trait_floor(&mut conn, collection_slug, token_id)
            .await?
            .0;
        let most_valuable_trait =
            get_most_valued_trait_floor(&mut conn, collection_slug, token_id, 0.025)
                .await?
                .0;
        let most_rare_trait_floor = Some(
            get_rarest_trait_floor(&mut conn, collection_slug, token_id)
                .await?
                .2,
        );
        let most_valued_trait_floor =
            get_most_valued_trait_floor(&mut conn, collection_slug, token_id, 0.025)
                .await?
                .1;

        let rarity_weighted_floor =
            Some(get_rarity_weighted_floor(&mut conn, collection_slug, token_id, 0.01).await?);
        let last_sale = get_last_sale_price(&mut conn, collection_slug, token_id).await?;
        let avg_last_three_mvt_sales = get_most_valued_trait_last_sale_avg(
            &mut conn,
            collection_slug,
            token_id,
            0.025,
            Some(3),
        )
        .await?;
        let last_sale_relative_avg_collection_price =
            get_last_sale_relative_to_collection_avg(&mut conn, collection_slug, token_id).await?;
        let last_sale_relative_avg_mvt_price =
            get_last_sale_relative_to_mvt_avg(&mut conn, collection_slug, token_id, 0.025).await?;

        let mut prices = vec![
            collection_floor,
            last_sale.unwrap_or(0f64),
            most_rare_trait_floor.unwrap_or(0f64),
            most_valued_trait_floor.unwrap_or(0f64),
            rarity_weighted_floor.unwrap_or(0f64),
            avg_last_three_mvt_sales.unwrap_or(0f64),
            last_sale_relative_avg_collection_price.unwrap_or(0f64),
            last_sale_relative_avg_mvt_price.unwrap_or(0f64),
        ];

        prices.sort_by(|a, b| b.partial_cmp(&a).unwrap());

        Ok(Self {
            collection_slug: collection_slug.to_string(),
            token_id,
            rarest_trait,
            most_valuable_trait,
            collection_floor,
            listing_price,
            last_sale,
            most_rare_trait_floor,
            most_valued_trait_floor,
            rarity_weighted_floor,
            avg_last_three_mvt_sales,
            last_sale_relative_avg_collection_price,
            last_sale_relative_avg_mvt_price,
            max_price: prices[0],
        })
    }
}
