use local::api::server;
use local::updater::{update_collection_listings, update_collection_sales};
#[tokio::main]
async fn main() {
    //env_logger::Builder::from_default_env();
    server::start().await
}
