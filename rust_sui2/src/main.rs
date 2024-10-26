use rust_sui2::publish;

#[tokio::main]
async fn main() {
    publish().await.unwrap_or_else(|e| println!("{e}"));
}