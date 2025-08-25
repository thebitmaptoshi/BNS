This is the setup to run the Bitmap Indexer as a seperate overlay to the ORD client. Requires running Bitcoin Core, Ord client and Bitmap Indexer

# Bitmap Indexer

A standalone Rust crate for indexing Bitmap and BNS inscriptions as an overlay for the `ord` client (version 0.22.0). It stores inscriptions in SQLite, minimizes `ord server` API calls, and uses `libp2p` for decentralized synchronization.

## Prerequisites
- **Rust**: Install Rust and Cargo (version 1.80 or later) via [rustup](https://rustup.rs/).
- **Bitcoin Core**: Version 27.1, running on mainnet or testnet with RPC enabled.
- **ord**: Version 0.22.0, running with `ord server` for API access.
- **SQLite**: Included with `rusqlite` dependency.

## Setup Instructions
1. **Clone the Repository**:
   ```bash
   git clone https://github.com/your-repo/bitmap-indexer.git
   cd bitmap-indexer

Install Dependencies:
   ```bash
cargo build --release
   ```

Configure Bitcoin Core:
Ensure Bitcoin Core 27.1 is running with RPC enabled.
Edit bitcoin.conf (typically in ~/.bitcoin/bitcoin.conf):
   ```conf
rpcuser=your-username
rpcpassword=your-password
server=1
[mainnet]
rpcport=8332
   ```

For testnet, add testnet=1 and use rpcport=18332.

Configure ord:Install ord 0.22.0 (follow instructions at ordinals/ord).
Start ord server:
   ```bash
ord server --http-port 80
   ```

Ensure ord’s config file (~/.ord/config.toml) is set up:
   ```toml
chain = "mainnet"  # or "testnet"
bitcoin_rpc_url = "http://localhost:8332"  # or 18332 for testnet
bitcoin_rpc_username = "your-username"
bitcoin_rpc_password = "your-password"
data_dir = "~/.ord"
   ```

Configure Bitmap Indexer:
Create or edit ~/.ord/config.toml to include the [bitmap] section:
   ```toml
[bitmap]
cache_blocks = 144
validate_sat = false
parallelism_enabled = true
batch_size = 100
bns_history_mode = "prune"
bootstrap_nodes = []
   ```
Adjust bootstrap_nodes if running in a libp2p network with known peers.

Run the Indexer:
   ```bash
cargo run --release -- --config ~/.ord/config.toml
   ```
The indexer will sync from block 792,435, process new blocks, and broadcast updates via libp2p.

Verify Operation:Check SQLite databases in ~/.ord/bitmap_mainnet.db or ~/.ord/bitmap_testnet.db.
Monitor logs for inscription processing and libp2p events.

Notes
Ensure ord server is running on http://0.0.0.0:80.
For testnet, set chain = "testnet" in config.toml.
The indexer stores Bitmap and BNS data indefinitely, with BNS transfer data cached for 144 blocks.


   







