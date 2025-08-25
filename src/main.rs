use bitmap_indexer::{Config, BitmapIndexer};
use env_logger;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = Config::load_config("~/.ord/config.toml");
    let network_name = config.chain.clone();
    let mut indexer = BitmapIndexer::new(config, network_name);
    indexer.run().await;
}
