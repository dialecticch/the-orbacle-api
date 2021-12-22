use crate::analyzers::prices::*;
use crate::analyzers::velocity::*;
use crate::storage::{establish_connection, read::read_collection};
use anyhow::Result;

#[derive(serde::Serialize, Debug)]
pub struct PriceProfile {
    pub collection_floor: f64,
    pub last_sale: Option<f64>,
    pub most_rare_trait_floor: Option<f64>,
    pub most_valued_trait_floor: Option<f64>,
    pub rarity_weighted_floor: f64,
    pub avg_last_three_mvt_sales: Option<f64>,
    pub last_sale_relative_collection_avg: Option<f64>,
    pub last_sale_relative_mvt_avg: Option<f64>,
    pub max_price: f64,
}
#[derive(serde::Serialize, Debug)]
pub struct VelocityProfile {
    pub rarest_trait_sale_frequency_30d: f64,
    pub mvt_sale_frequency_30d: f64,
    pub lowest_sale_frequency_30d: f64,
    pub avg_sale_frequency_30d: f64,
}

#[derive(serde::Serialize, Debug)]
pub struct TokenProfile {
    pub opensea: String,
    pub collection_slug: String,
    pub token_id: i32,
    pub listing_price: Option<f64>,
    pub rarest_trait: String,
    pub most_valuable_trait: Option<String>,
    pub price_profile: PriceProfile,
    pub velocity_profile: VelocityProfile,
}

impl TokenProfile {
    pub async fn make(collection_slug: &str, token_id: i32, cutoff: i32) -> Result<Self> {
        let pool = establish_connection().await;
        let mut conn = pool.acquire().await?;

        let collection_address = read_collection(&mut conn, collection_slug).await?.address;
        let collection_floor = get_collection_floor(collection_slug).await?;

        let listing_price = get_token_listing(&mut conn, collection_slug, token_id).await?;
        let rarest_trait = get_rarest_trait_floor(&mut conn, collection_slug, token_id)
            .await?
            .0;
        let most_valuable_trait =
            get_most_valued_trait_floor(&mut conn, collection_slug, token_id, 1f64)
                .await?
                .0;
        let most_rare_trait_floor = get_rarest_trait_floor(&mut conn, collection_slug, token_id)
            .await?
            .2;
        let most_valued_trait_floor =
            get_most_valued_trait_floor(&mut conn, collection_slug, token_id, 1f64)
                .await?
                .1;

        let rarity_weighted_floor =
            get_rarity_weighted_floor(&mut conn, collection_slug, token_id, cutoff as f64 / 100f64)
                .await?;
        let last_sale = get_last_sale_price(&mut conn, collection_slug, token_id).await?;
        let avg_last_three_mvt_sales = get_most_valued_trait_last_sale_avg(
            &mut conn,
            collection_slug,
            token_id,
            1f64,
            Some(3),
        )
        .await?;
        let last_sale_relative_collection_avg =
            get_last_sale_relative_to_collection_avg(&mut conn, collection_slug, token_id).await?;
        let last_sale_relative_mvt_avg =
            get_last_sale_relative_to_mvt_avg(&mut conn, collection_slug, token_id, 1f64).await?;

        let avg_sale_frequency_30d =
            get_avg_sale_frequency(&mut conn, collection_slug, token_id, 30).await?;
        let lowest_sale_frequency_30d =
            get_lowest_sale_frequency(&mut conn, collection_slug, token_id, 30).await?;
        let mvt_sale_frequency_30d = get_sale_frequency_trait(
            &mut conn,
            collection_slug,
            &most_valuable_trait.clone().unwrap(),
            30,
        )
        .await?;

        let rarest_trait_sale_frequency_30d =
            get_sale_frequency_trait(&mut conn, collection_slug, &rarest_trait.clone(), 30).await?;

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
            opensea: format!(
                "https://opensea.io/assets/{}/{}",
                collection_address, token_id
            ),
            collection_slug: collection_slug.to_string(),
            token_id,
            rarest_trait,
            most_valuable_trait,
            listing_price,
            price_profile: PriceProfile {
                collection_floor,
                last_sale,
                most_rare_trait_floor,
                most_valued_trait_floor,
                rarity_weighted_floor,
                avg_last_three_mvt_sales,
                last_sale_relative_collection_avg,
                last_sale_relative_mvt_avg,
                max_price: prices[0],
            },
            velocity_profile: VelocityProfile {
                lowest_sale_frequency_30d: lowest_sale_frequency_30d.1,
                rarest_trait_sale_frequency_30d,
                avg_sale_frequency_30d,
                mvt_sale_frequency_30d,
            },
        })
    }
}
