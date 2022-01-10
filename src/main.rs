use governor::{Quota, RateLimiter};
use local::api::server;
use local::storage::establish_connection;
use local::updater::update::update_db;

#[tokio::main]
async fn main() {
    // for github app
    let pool = establish_connection().await;
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // run the updater in the background
    let lim =
        RateLimiter::direct(Quota::with_period(std::time::Duration::from_secs(3600u64)).unwrap());
    tokio::task::spawn(update_db(lim));

    //start the server
    server::start().await
}
