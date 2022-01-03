use super::errors::handle_rejection;
use super::handlers;
use crate::storage::establish_connection;
use rweb::*;

#[allow(clippy::redundant_clone)] // clippy bug- clones are not redundant
pub async fn start() {
    let pool = establish_connection().await;
    let cors = warp::cors()
        .allow_methods(vec!["GET", "POST", "PATCH", "DELETE"])
        .allow_headers(["Authorization", "Content-Type"])
        .max_age(86400);

    let endpoint = dotenv::var("ENDPOINT").unwrap();
    let port = dotenv::var("PORT").unwrap().parse::<u16>().unwrap();
    let (spec, filter) = openapi::spec().build(move || {
        warp::any()
            .and(handlers::status(pool.clone()).boxed())
            .or(handlers::get_profile(pool.clone()).boxed())
            .or(handlers::get_price_profile(pool.clone()).boxed())
            .or(handlers::get_collection_profile(pool.clone()).boxed())
            .or(handlers::get_wallet_profile(pool.clone()).boxed())
            .or(handlers::get_all_collections(pool.clone()).boxed())
            .recover(handle_rejection)
            .with(cors)
    });

    serve(filter.or(openapi_docs(spec)))
        .run((get_endpoint(&endpoint), port))
        .await;
}

pub fn get_endpoint(address: &str) -> [u8; 4] {
    address
        .split('.')
        .map(|s| s.parse().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
