use local::api::server;
#[tokio::main]
async fn main() {
    server::start().await
}
