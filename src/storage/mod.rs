pub mod read;
pub mod write;

#[derive(serde::Serialize, Debug)]
pub struct Trait {
    collection_slug: String,
    trait_type: String,
    trait_name: String,
    trait_count: i32,
}
#[derive(serde::Serialize, Debug)]
pub struct Asset {
    collection_slug: String,
    pub token_id: i32,
    traits: Vec<String>,
}

#[derive(serde::Serialize, Debug)]
pub struct Collection {
    pub slug: String,
    pub address: String,
    pub total_supply: i32,
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
