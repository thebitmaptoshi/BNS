These are the changes to the ord files needed to implement a soft fork of the ord client and run this instead of ord... All the same info, but plus bitmap tracking and BNS registry


### Integrated `ord` Fork README (Bitmap Subcommand)

markdown
# ord with Bitmap Subcommand

This fork of `ord` (version 0.22.0) adds a `bitmap` subcommand for indexing Bitmap and BNS inscriptions, storing data in SQLite and using `libp2p` for decentralized synchronization. The `bitmap` feature is optional and minimally impacts the core `ord` codebase.

## Prerequisites
- **Rust**: Install Rust and Cargo (version 1.65 or later) via [rustup](https://rustup.rs/).
- **Bitcoin Core**: Version 27.1 or later, running with RPC enabled.
- **ord**: This fork of `ord` 0.22.0 (ensure you’re using this repository).
- **SQLite**: Included with `rusqlite` dependency when `bitmap` feature is enabled.

## Setup Instructions
1. **Clone the Forked Repository**:
   ```bash
   git clone https://github.com/your-repo/ord.git
   cd ord
    ```
Install Dependencies:Build with the bitmap feature:
   ```bash
cargo build --release --features bitmap
   ```
This includes rusqlite, serde_cbor, reqwest, and libp2p for the bitmap subcommand.

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

Configure ord:
Edit ~/.ord/config.toml to include Bitcoin RPC and bitmap settings:
   ```toml
chain = "mainnet"  # or "testnet"
bitcoin_rpc_url = "http://localhost:8332"  # or 18332 for testnet
bitcoin_rpc_username = "your-username"
bitcoin_rpc_password = "your-password"
data_dir = "~/.ord"

[bitmap]
cache_blocks = 144
validate_sat = false
parallelism_enabled = true
batch_size = 100
bns_history_mode = "prune"
bootstrap_nodes = []
   ```

Adjust bootstrap_nodes for libp2p peers if needed.

Run ord Server:Start ord server for API access:
   ```bash
cargo run --release --features bitmap -- server --http-port 80
   ```
Run the Bitmap Subcommand:
In a separate terminal, run the bitmap subcommand:
   ```bash
cargo run --release --features bitmap -- bitmap
   ```
The subcommand syncs from block 792,435, processes new blocks, and broadcasts updates via libp2p.

Verify Operation:Check SQLite databases in ~/.ord/bitmap_mainnet.db or ~/.ord/bitmap_testnet.db.
Monitor logs for inscription processing and libp2p events.

Notes
The bitmap feature is optional; build without --features bitmap to exclude it.
Ensure ord server is running on http://0.0.0.0:80.
For testnet, set chain = "testnet" in config.toml.
Bitmap and BNS data are stored indefinitely, with BNS transfer data cached for 144 blocks.






