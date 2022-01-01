pub mod preprocess;
pub mod read;
pub mod write;

#[derive(serde::Serialize, Debug)]
pub struct Trait {
    pub collection_slug: String,
    pub trait_type: String,
    pub trait_name: String,
    pub trait_count: i32,
}
#[derive(serde::Serialize, Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub collection_slug: String,
    pub token_id: i32,
    pub image_url: String,
    pub owner: String,
    pub traits: Vec<String>,
    pub unique_traits: i32,
    pub traits_3_combination_overlap: i32,
    pub traits_4_combination_overlap: i32,
    pub traits_5_combination_overlap: i32,
    pub traits_3_combination_overlap_ids: Vec<i32>,
    pub traits_4_combination_overlap_ids: Vec<i32>,
    pub traits_5_combination_overlap_ids: Vec<i32>,
}

#[derive(serde::Serialize, Debug)]
pub struct Collection {
    pub slug: String,
    pub address: String,
    pub total_supply: i32,
    pub floor_price: f64,
    pub rarity_cutoff: f64,
    pub banner_image_url: String,
    pub daily_volume: f64,
    pub daily_sales: f64,
    pub daily_avg_price: f64,
    pub weekly_avg_price: f64,
    pub monthly_avg_price: f64,
    pub nr_owners: f64,
    pub avg_trait_rarity: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SaleEvent {
    pub collection_slug: String,
    pub token_id: i32,
    pub timestamp: i32,
    pub price: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Listing {
    pub collection_slug: String,
    pub update_type: String,
    pub token_id: i32,
    pub timestamp: i32,
    pub price: Option<f64>,
}

use dotenv::dotenv;
use sqlx::PgPool;
use std::env;

pub async fn establish_connection() -> PgPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect(&database_url)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
