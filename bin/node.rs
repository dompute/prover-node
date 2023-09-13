use prover_node::listen_on;

#[tokio::main]
async fn main() {
    let url = "ws://localhost:8545";
    listen_on(url).await.unwrap();
}
