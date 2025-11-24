# Setting Up Zebra Node

Quick guide to download and run Zebra for testing.

## Option 1: Download Pre-Built Binary (Fastest)

### Windows

1. **Download Zebra:**
   ```bash
   # Go to releases page
   https://github.com/ZcashFoundation/zebra/releases

   # Download latest Windows binary
   # Look for: zebra-windows-x86_64-v*.zip
   ```

2. **Extract and run:**
   ```powershell
   # Extract the zip
   # Open PowerShell in that folder

   # Run Zebra
   .\zebrad.exe start
   ```

### Linux/Mac

```bash
# Download latest release
curl -L https://github.com/ZcashFoundation/zebra/releases/latest/download/zebrad -o zebrad

# Make executable
chmod +x zebrad

# Run it
./zebrad start
```

## Option 2: Install via Cargo (Recommended)

```bash
# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Zebra
cargo install --locked zebrad

# Verify installation
zebrad --version

# Should output: zebrad 1.x.x
```

## Running Zebra

### Start Syncing

```bash
# Start with default config (mainnet)
zebrad start

# Or for testnet (faster sync)
zebrad start --network testnet
```

### What You'll See

```
Initializing Zebra...
Starting sync...
Block 1,000 / 2,450,123 (0.04%)
Block 10,000 / 2,450,123 (0.4%)
Block 50,000 / 2,450,123 (2.0%)
...
```

**Sync Time:**
- **Testnet:** ~2-4 hours
- **Mainnet:** ~12-48 hours (depends on hardware)

**Storage:**
- **Testnet:** ~10-20 GB
- **Mainnet:** ~50-100 GB

## Where Data is Stored

**Default locations:**
- **Linux/Mac:** `~/.zebra/`
- **Windows:** `C:\Users\[username]\.zebra\`

**Directory structure:**
```
.zebra/
├── zebra.toml          # Config file
└── state/
    ├── mainnet/
    │   └── db/         # RocksDB database (blockchain data)
    └── testnet/
        └── db/
```

## Quick Commands

```bash
# Check status
zebrad status

# Check version
zebrad --version

# Stop Zebra (Ctrl+C in terminal)

# View logs
tail -f ~/.zebra/zebra.log
```

## Testing Our Proof Generator

Once Zebra is syncing, you can test reading its data:

```bash
# Check Zebra's state
ls -lh ~/.zebra/state/testnet/db/

# Should see RocksDB files
```

## Troubleshooting

### "Port already in use"
```bash
# Zebra uses port 8233 (mainnet) or 18233 (testnet)
# Kill any existing process
pkill zebrad
```

### Slow sync
```bash
# Use testnet for faster testing
zebrad start --network testnet

# Or use checkpoint sync (faster initial sync)
# This is automatic in newer versions
```

### Out of disk space
```bash
# Check available space
df -h

# Need at least:
# - Testnet: 20 GB free
# - Mainnet: 120 GB free
```

## Config File

Create custom config:

```bash
# Generate default config
zebrad generate -o zebra.toml

# Edit it
nano zebra.toml

# Run with custom config
zebrad -c zebra.toml start
```

## Example Config (zebra.toml)

```toml
[network]
network = "Testnet"  # or "Mainnet"

[state]
cache_dir = "~/.zebra"

[tracing]
filter = "info"

[rpc]
listen_addr = "127.0.0.1:8232"
```

## Next Steps

Once Zebra is syncing:

1. ✅ Let it sync (can take hours)
2. ✅ Check the database structure
3. ✅ Read data with our proof generator
4. ✅ Generate test proofs

## Useful Links

- [Zebra Docs](https://zebra.zfnd.org/)
- [GitHub](https://github.com/ZcashFoundation/zebra)
- [Zcash Foundation](https://www.zfnd.org/)
