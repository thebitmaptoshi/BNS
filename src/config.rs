use config::{Config as ConfigLoader, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub chain: String,
    pub bitcoin_rpc_url: String,
    pub bitcoin_rpc_username: String,
    pub bitcoin_rpc_password: String,
    pub data_dir: String,
    pub bitmap: BitmapConfig,
}

#[derive(Deserialize, Debug)]
pub struct BitmapConfig {
    #[serde(default = "default_cache_blocks")]
    pub cache_blocks: usize,
    #[serde(default = "default_validate_sat")]
    pub validate_sat: bool,
    #[serde(default = "default_parallelism_enabled")]
    pub parallelism_enabled: bool,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_bns_history_mode")]
    pub bns_history_mode: String,
    #[serde(default)]
    pub bootstrap_nodes: Vec<String>,
}

fn default_cache_blocks() -> usize { 144 }
fn default_validate_sat() -> bool { false }
fn default_parallelism_enabled() -> bool { true }
fn default_batch_size() -> usize { 100 }
fn default_bns_history_mode() -> String { "prune".to_string() }

pub fn load_config(config_path: &str) -> Config {
    ConfigLoader::builder()
        .add_source(File::with_name(config_path))
        .set_default("bitmap.cache_blocks", 144).unwrap()
        .set_default("bitmap.validate_sat", false).unwrap()
        .set_default("bitmap.parallelism_enabled", true).unwrap()
        .set_default("bitmap.batch_size", 100).unwrap()
        .set_default("bitmap.bns_history_mode", "prune").unwrap()
        .set_default("bitmap.bootstrap_nodes", vec![] as Vec<String>).unwrap()
        .build()
        .expect("Failed to load config")
        .try_deserialize::<Config>()
        .expect("Failed to deserialize config")
}
