use local::api::server;
use local::updater::update::update_db;

use governor::{Quota, RateLimiter};

#[tokio::main]
async fn main() {
    // run the updater in the background
    let lim =
        RateLimiter::direct(Quota::with_period(std::time::Duration::from_secs(3600u64)).unwrap());
    //tokio::task::spawn(update_db(lim));

    //start the server
    server::start().await
}
