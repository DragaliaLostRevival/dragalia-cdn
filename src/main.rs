mod dragalia_cdn;

#[tokio::main]
async fn main() {
    dragalia_cdn::server::start_server().await;
}