use crate::database::{BitmapEntry, BnsEntry, Database};
use crate::network::{BitmapMessage, Network};
use bitcoin::rpc::Client as BitcoinClient;
use ordinals::{Inscription, SatPoint};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use rayon::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct InscriptionData {
    pub id: String,
    pub sat_point: SatPoint,
    pub content: String,
    pub owner: String,
    pub timestamp: u64,
    pub inscription_block: u64,
    pub tx_index: u64,
    pub children: Vec<InscriptionChild>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InscriptionChild {
    pub id: String,
}

pub struct BitmapCache {
    bns_entries: VecDeque<(u64, BnsEntry)>,
    max_size: usize,
}

impl BitmapCache {
    pub fn new(max_size: usize) -> Self {
        BitmapCache {
            bns_entries: VecDeque::new(),
            max_size,
        }
    }

    pub fn add_bns_entry(&mut self, block_height: u64, entry: BnsEntry) {
        if entry.transfer_status {
            self.bns_entries.push_back((block_height, entry));
            while self.bns_entries.len() > self.max_size {
                self.bns_entries.pop_front();
            }
        }
    }

    pub fn check_cooldown(&self, blockheight: &str, current_block: u64) -> bool {
        for (block_height, entry) in &self.bns_entries {
            if entry.address == blockheight && entry.transfer_status && current_block - block_height < 144 {
                return false;
            }
        }
        true
    }
}

pub struct BitmapIndexer {
    config: crate::config::BitmapConfig,
    db: Database,
    cache: BitmapCache,
    network: Network,
    rpc: BitcoinClient,
    http_client: Client,
    current_block_height: u64,
    network_name: String,
}

impl BitmapIndexer {
    pub fn new(config: crate::config::Config, network_name: String) -> Self {
        let rpc = BitcoinClient::new(
            &config.bitcoin_rpc_url,
            &config.bitcoin_rpc_username,
            &config.bitcoin_rpc_password,
        ).unwrap();
        let db = Database::new(&config.data_dir, &network_name);
        let cache = BitmapCache::new(config.bitmap.cache_blocks);
        let network = Network::new(identity::Keypair::generate_ed25519(), config.bitmap.bootstrap_nodes, &network_name);
        BitmapIndexer {
            config: config.bitmap,
            db,
            cache,
            network,
            rpc,
            http_client: Client::new(),
            current_block_height: 792435,
            network_name,
        }
    }

    pub async fn run(&mut self) {
        self.sync_blocks(792435, self.rpc.get_block_count().unwrap()).await;
        loop {
            let new_height = self.rpc.get_block_count().unwrap();
            if new_height > self.current_block_height {
                self.process_block(new_height).await;
                self.current_block_height = new_height;
                self.db.prune_bns_history(new_height, &self.config.bns_history_mode, self.config.cache_blocks);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    async fn sync_blocks(&mut self, start_height: u64, end_height: u64) {
        for height in (start_height..=end_height).step_by(self.config.batch_size) {
            let response = self.http_client
                .get(format!("http://0.0.0.0:80/inscriptions/block/{}", height))
                .header("Accept", "application/json")
                .send()
                .await
                .unwrap()
                .json::<Vec<InscriptionData>>()
                .await
                .unwrap();
            self.process_inscriptions(height, response).await;
        }
    }

    async fn process_block(&mut self, block_height: u64) {
        let response = self.http_client
            .get(format!("http://0.0.0.0:80/inscriptions/block/{}", block_height))
            .header("Accept", "application/json")
            .send()
            .await
            .unwrap()
            .json::<Vec<InscriptionData>>()
            .await
            .unwrap();
        self.process_inscriptions(block_height, response).await;
    }

    async fn process_inscriptions(&mut self, block_height: u64, inscriptions: Vec<InscriptionData>) {
        let inscriptions_chunks = inscriptions.chunks(self.config.batch_size);
        if self.config.parallelism_enabled {
            inscriptions_chunks.par_bridge().for_each(|chunk| {
                for inscription in chunk {
                    self.process_bitmap_inscription(block_height, inscription);
                    self.process_bns_inscription(block_height, inscription);
                }
            });
        } else {
            for inscription in inscriptions {
                self.process_bitmap_inscription(block_height, &inscription);
                self.process_bns_inscription(block_height, &inscription);
            }
        }
    }

    fn process_bitmap_inscription(&self, block_height: u64, inscription: &InscriptionData) {
        if !inscription.content.ends_with(".bitmap") {
            return;
        }
        let parts: Vec<&str> = inscription.content.split('.').collect();
        if parts.len() != 2 && parts.len() != 3 {
            return;
        }
        let is_parcel = parts.len() == 3;
        let blockheight_num = parts[if is_parcel { 1 } else { 0 }].parse::<u64>().unwrap_or(0);
        if block_height < blockheight_num || block_height < 792435 {
            return;
        }
        let target_blockheight = if is_parcel {
            format!("{}.{}", parts[0], parts[1])
        } else {
            parts[0].to_string()
        };

        if self.db.conn.query_row(
            "SELECT inscription_id FROM bitmap_registry WHERE blockheight = ?",
            params![&target_blockheight],
            |row| row.get::<_, String>(0),
        ).is_ok() {
            return; // Duplicate
        }

        if is_parcel {
            let district_blockheight = parts[1];
            if let Ok(district) = self.db.conn.query_row(
                "SELECT inscription_id FROM bitmap_registry WHERE blockheight = ?",
                params![district_blockheight],
                |row| row.get::<_, String>(0),
            ) {
                let response = self.http_client
                    .get(format!("http://0.0.0.0:80/inscription/{}", district))
                    .header("Accept", "application/json")
                    .block_on()
                    .unwrap()
                    .json::<InscriptionData>()
                    .block_on()
                    .unwrap();
                let is_child = response.children.iter().any(|child| child.id == inscription.id && inscription.content == target_blockheight);
                if !is_child {
                    return; // Invalid parcel
                }
            } else {
                return; // District not found
            }
        }

        let entry = BitmapEntry {
            blockheight: target_blockheight.clone(),
            timestamp: inscription.timestamp,
            inscription_id: inscription.id.clone(),
            satpoint: serde_json::to_string(&inscription.sat_point).unwrap(),
            current_owner: inscription.owner.clone(),
            transfer_block: block_height,
        };
        self.db.store_bitmap_entry(&entry);
        self.network.broadcast_message(&BitmapMessage::BitmapRegistration { entry });
    }

    fn process_bns_inscription(&self, block_height: u64, inscription: &InscriptionData) {
        let is_bns = match serde_cbor::from_slice::<serde_json::Value>(inscription.content.as_bytes()) {
            Ok(json) => json.get("BNS").and_then(|v| v.as_bool()).unwrap_or(false),
            Err(_) => false,
        };
        if !is_bns {
            return;
        }

        if let Err(err) = self.validate_bns_inscription(block_height, inscription) {
            if self.network_name == "testnet" {
                let pending_entry = BnsEntry {
                    name: inscription.content.clone(),
                    address: "".to_string(),
                    owner: inscription.owner.clone(),
                    inscription_block: inscription.inscription_block,
                    tx_index: inscription.tx_index,
                    inscription_id: inscription.id.clone(),
                    sat_number: inscription.sat_point.sat,
                    transfer_status: false,
                    transfer_block: block_height,
                    timestamp: inscription.timestamp,
                    original_blockheight: "".to_string(),
                    previous_owner: None,
                    previous_inscription_id: None,
                    previous_sat_number: None,
                };
                self.db.store_bns_entry(&pending_entry, true);
            }
            log::error!("Invalid BNS inscription: {}", err);
            return;
        }

        let is_parcel = inscription.content.chars().next().unwrap().is_numeric() && inscription.content.contains('.');
        let address = if is_parcel {
            inscription.content.split('.').nth(1).unwrap().split('.').next().unwrap().to_string()
        } else {
            inscription.content.split('.').next().unwrap().to_string()
        };

        let entry = BnsEntry {
            name: inscription.content.clone(),
            address,
            owner: inscription.owner.clone(),
            inscription_block: inscription.inscription_block,
            tx_index: inscription.tx_index,
            inscription_id: inscription.id.clone(),
            sat_number: inscription.sat_point.sat,
            transfer_status: false,
            transfer_block: block_height,
            timestamp: inscription.timestamp,
            original_blockheight: address.clone(),
            previous_owner: None,
            previous_inscription_id: None,
            previous_sat_number: None,
        };
        self.db.store_bns_entry(&entry, self.network_name == "testnet");
        self.cache.add_bns_entry(block_height, entry.clone());
        self.network.broadcast_message(&BitmapMessage::BnsInscription { entry });
    }

    fn validate_bns_inscription(&self, block_height: u64, inscription: &InscriptionData) -> Result<(), String> {
        let bitmap = self.db.conn.query_row(
            "SELECT blockheight, current_owner FROM bitmap_registry WHERE satpoint = ?",
            params![serde_json::to_string(&inscription.sat_point).unwrap()],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        ).map_err(|_| "Bitmap not found".to_string())?;

        if inscription.content.chars().next().unwrap().is_numeric() {
            let expected_name = format!("{}.bitmap", bitmap.0);
            if inscription.content != expected_name {
                return Err("numeric name does not match registered Bitmap blockheight".to_string());
            }
            if inscription.content.contains('.') {
                let district_blockheight = inscription.content.split('.').nth(1).unwrap().split('.').next().unwrap();
                let district = self.db.conn.query_row(
                    "SELECT inscription_id FROM bitmap_registry WHERE blockheight = ?",
                    params![district_blockheight],
                    |row| row.get::<_, String>(0),
                ).map_err(|_| "District not found".to_string())?;
                let response = self.http_client
                    .get(format!("http://0.0.0.0:80/inscription/{}", district))
                    .header("Accept", "application/json")
                    .block_on()
                    .unwrap()
                    .json::<InscriptionData>()
                    .block_on()
                    .unwrap();
                let is_child = response.children.iter().any(|child| child.id == inscription.id && inscription.content == expected_name);
                if !is_child {
                    return Err("Parcel not a child of district".to_string());
                }
            }
        }

        if !self.cache.check_cooldown(&bitmap.0, block_height) {
            return Err("BNS cooldown violation".to_string());
        }

        if self.config.validate_sat {
            let response = self.http_client
                .get(format!("http://0.0.0.0:80/sat/{}", inscription.sat_point.sat))
                .header("Accept", "application/json")
                .block_on()
                .unwrap()
                .json::<serde_json::Value>()
                .block_on()
                .unwrap();
            if response.get("owner").and_then(|v| v.as_str()) != Some(&inscription.owner) {
                return Err("Satoshi ownership mismatch".to_string());
            }
        }

        Ok(())
    }

    fn resolve_timestamp_mismatch(&self, inscription_id: &str, local_timestamp: u64) -> u64 {
        self.network.broadcast_message(&BitmapMessage::TimestampRequest { inscription_id: inscription_id.to_string() });
        // Collect responses, return majority timestamp
        local_timestamp // Placeholder
    }
}
