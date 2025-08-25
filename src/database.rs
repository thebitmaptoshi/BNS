use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct BitmapEntry {
    pub blockheight: String,
    pub timestamp: u64,
    pub inscription_id: String,
    pub satpoint: String,
    pub current_owner: String,
    pub transfer_block: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BnsEntry {
    pub name: String,
    pub address: String,
    pub owner: String,
    pub inscription_block: u64,
    pub tx_index: u64,
    pub inscription_id: String,
    pub sat_number: u64,
    pub transfer_status: bool,
    pub transfer_block: u64,
    pub timestamp: u64,
    pub original_blockheight: String,
    pub previous_owner: Option<String>,
    pub previous_inscription_id: Option<String>,
    pub previous_sat_number: Option<u64>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(data_dir: &str, network: &str) -> Self {
        let db_path = Path::new(data_dir).join(format!("bitmap_{}.db", network));
        let conn = Connection::open(db_path).expect("Failed to open SQLite database");
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bitmap_registry (
                blockheight TEXT PRIMARY KEY,
                timestamp INTEGER,
                inscription_id TEXT,
                satpoint TEXT,
                current_owner TEXT,
                transfer_block INTEGER
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bitmap_bns_registry (
                name TEXT PRIMARY KEY,
                address TEXT,
                owner TEXT,
                inscription_block INTEGER,
                tx_index INTEGER,
                inscription_id TEXT,
                sat_number INTEGER,
                transfer_status BOOLEAN,
                transfer_block INTEGER,
                timestamp INTEGER,
                original_blockheight TEXT,
                previous_owner TEXT,
                previous_inscription_id TEXT,
                previous_sat_number INTEGER
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bitmap_bns_history (
                name TEXT,
                address TEXT,
                owner TEXT,
                inscription_block INTEGER,
                tx_index INTEGER,
                inscription_id TEXT,
                sat_number INTEGER,
                transfer_status BOOLEAN,
                transfer_block INTEGER,
                timestamp INTEGER,
                original_blockheight TEXT,
                previous_owner TEXT,
                previous_inscription_id TEXT,
                previous_sat_number INTEGER
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bitmap_pending_review (
                inscription_id TEXT PRIMARY KEY,
                name TEXT,
                address TEXT,
                owner TEXT,
                inscription_block INTEGER,
                tx_index INTEGER,
                sat_number INTEGER,
                reason TEXT
            )",
            [],
        ).unwrap();
        Database { conn }
    }

    pub fn store_bitmap_entry(&self, entry: &BitmapEntry) {
        self.conn.execute(
            "INSERT OR REPLACE INTO bitmap_registry (blockheight, timestamp, inscription_id, satpoint, current_owner, transfer_block)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                &entry.blockheight,
                entry.timestamp,
                &entry.inscription_id,
                &entry.satpoint,
                &entry.current_owner,
                entry.transfer_block
            ],
        ).unwrap();
    }

    pub fn store_bns_entry(&self, entry: &BnsEntry, is_testnet: bool) {
        self.conn.execute(
            "INSERT OR REPLACE INTO bitmap_bns_registry (
                name, address, owner, inscription_block, tx_index, inscription_id, sat_number,
                transfer_status, transfer_block, timestamp, original_blockheight,
                previous_owner, previous_inscription_id, previous_sat_number
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &entry.name, &entry.address, &entry.owner, entry.inscription_block, entry.tx_index,
                &entry.inscription_id, entry.sat_number, entry.transfer_status, entry.transfer_block,
                entry.timestamp, &entry.original_blockheight, &entry.previous_owner,
                &entry.previous_inscription_id, &entry.previous_sat_number
            ],
        ).unwrap();
        if entry.transfer_status {
            self.conn.execute(
                "INSERT INTO bitmap_bns_history (
                    name, address, owner, inscription_block, tx_index, inscription_id, sat_number,
                    transfer_status, transfer_block, timestamp, original_blockheight,
                    previous_owner, previous_inscription_id, previous_sat_number
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    &entry.name, &entry.address, &entry.owner, entry.inscription_block, entry.tx_index,
                    &entry.inscription_id, entry.sat_number, entry.transfer_status, entry.transfer_block,
                    entry.timestamp, &entry.original_blockheight, &entry.previous_owner,
                    &entry.previous_inscription_id, &entry.previous_sat_number
                ],
            ).unwrap();
        }
        if is_testnet && !entry.transfer_status {
            self.conn.execute(
                "INSERT OR REPLACE INTO bitmap_pending_review (
                    inscription_id, name, address, owner, inscription_block, tx_index, sat_number, reason
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    &entry.inscription_id, &entry.name, &entry.address, &entry.owner,
                    entry.inscription_block, entry.tx_index, entry.sat_number, "Invalid BNS"
                ],
            ).unwrap();
        }
    }

    pub fn prune_bns_history(&self, current_block: u64, bns_history_mode: &str, cache_blocks: usize) {
        if bns_history_mode == "prune" {
            self.conn.execute(
                "DELETE FROM bitmap_bns_history WHERE transfer_block < ?",
                params![current_block - cache_blocks as u64],
            ).unwrap();
        }
    }
}
