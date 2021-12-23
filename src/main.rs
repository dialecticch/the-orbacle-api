use local::api::server;
#[tokio::main]
async fn main() {
    //env_logger::Builder::from_default_env();
    server::start().await
}
