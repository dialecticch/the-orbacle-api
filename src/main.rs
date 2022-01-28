use local::api::server;
use local::storage::establish_connection;
use local::sync::sync_events::sync_events_loop;

#[tokio::main]
async fn main() {
    // for github app
    let pool = establish_connection().await;
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // run the updater in the background
    tokio::task::spawn(sync_events_loop());

    println!("Starting server...");

    //start the server
    server::start().await
}
