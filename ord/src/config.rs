use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    // Existing fields
    pub chain: String,
    pub bitcoin_rpc_url: String,
    pub bitcoin_rpc_username: String,
    pub bitcoin_rpc_password: String,
    pub data_dir: String,
    #[serde(default)]
    pub bitmap: BitmapConfig,
}

#[derive(Deserialize, Default)]
pub struct BitmapConfig {
    pub cache_blocks: Option<usize>,
    pub validate_sat: Option<bool>,
    pub parallelism_enabled: Option<bool>,
    pub batch_size: Option<usize>,
    pub bns_history_mode: Option<String>,
    pub bootstrap_nodes: Option<Vec<String>>,
}

// Minimal migration helpers for integration index tables
pub mod bitmap_integration_migrations {
    use rusqlite::Connection;

    pub fn create_bitmap_index_table(conn: &Connection) {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bitmap_index (
                blockheight TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                inscription_id TEXT NOT NULL,
                satpoint TEXT NOT NULL,
                current_owner TEXT NOT NULL,
                transfer_block INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();
    }

    pub fn create_bns_index_table(conn: &Connection) {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bns_index (
                name TEXT PRIMARY KEY,
                address TEXT NOT NULL,
                owner TEXT NOT NULL,
                inscription_block INTEGER NOT NULL,
                tx_index INTEGER NOT NULL,
                inscription_id TEXT NOT NULL,
                sat_number INTEGER NOT NULL,
                transfer_status INTEGER NOT NULL,
                transfer_block INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                is_default_name INTEGER NOT NULL,
                original_blockheight TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
    }
}