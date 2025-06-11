mod storage;
mod event;
mod server;

#[tokio::main]
async fn main() {
    server::serve().await.unwrap();
}

