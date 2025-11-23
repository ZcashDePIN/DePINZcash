# DePINZcash Quick Start Guide

Get started earning rewards for running a Zcash node in under 30 minutes!

## Prerequisites Check

Before starting, ensure you have:

- [ ] Computer with 100+ GB free space
- [ ] 8+ GB RAM (16 GB recommended)
- [ ] Stable internet connection
- [ ] Linux, macOS, or Windows

## Step 1: Install Zebra (15 minutes)

### Linux/macOS

```bash
# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Zebra
cargo install --locked zebrad

# Verify installation
zebrad --version
```

### Windows

Download the Windows installer from:
https://github.com/ZcashFoundation/zebra/releases

Or use cargo (requires Rust):
```powershell
cargo install --locked zebrad
```

## Step 2: Start Zebra Sync (6-24 hours)

```bash
# Start Zebra (it will begin syncing)
zebrad start

# This will take hours/days depending on your hardware
# You can close the terminal - Zebra runs in background

# Check sync progress (in new terminal)
zebrad status
```

**Tip:** Let Zebra sync overnight. You'll see progress like:
```
Syncing: block 1,234,567 / 2,450,000 (50.3%)
```

## Step 3: Install DePINZcash Prover (5 minutes)

While Zebra is syncing, set up the prover:

```bash
# Clone the repository
git clone https://github.com/depinzcash/DePINZcash
cd DePINZcash

# Run setup script
./scripts/setup.sh

# This will:
# - Build the Rust proof generator
# - Create necessary directories
# - Ask for your wallet addresses
```

When prompted, enter:
1. **Solana wallet address** (where you want to receive rewards)
2. **Zcash address** (t-addr or z-addr)
3. Optional node identifier (or press Enter)

## Step 4: Generate Your First Proof (2 minutes)

**Wait until Zebra is at least 75% synced!**

```bash
# Check if Zebra is ready
zebrad status

# Generate proof
./scripts/generate_proof.sh
```

You'll see output like:
```
🦓 DePINZcash Proof Generator
==============================

✓ Zebra found at: /home/user/.zebra
✓ Zebra binary verified

Reading node metrics...
  Block height: 2,450,123
  Sync progress: 100.00%
  Network: mainnet

Generating zero-knowledge proof...
(This may take 1-2 minutes)

✓ Proof generated successfully!
✓ Proof saved to: proofs/proof_1700000000.json

📤 Next step:
   Upload this file to: https://depinzcash.io/submit
```

## Step 5: Submit Proof & Get Paid (1 minute)

1. Go to https://depinzcash.io/submit
2. Upload the proof file: `proofs/proof_XXXXX.json`
3. Wait for verification (usually < 5 minutes)
4. Receive reward notification via email
5. Get paid weekly on Mondays!

## What's Next?

### Daily Operation

```bash
# Check Zebra is running
ps aux | grep zebrad

# Generate new proof (do this weekly)
./scripts/generate_proof.sh

# Submit proof to website
# (Manual for now, auto-submission coming soon)
```

### Maximize Earnings

✅ **Keep Zebra running 24/7** - Earn uptime rewards
✅ **Open port 8233** - Serve peers for 1.5x multiplier
✅ **Monitor sync status** - Ensure you stay synced
✅ **Submit weekly proofs** - Don't miss reward claims

### Port Forwarding (for peer serving bonus)

**Linux/macOS:**
```bash
# Check if port is open
sudo ufw allow 8233/tcp

# Or using iptables
sudo iptables -A INPUT -p tcp --dport 8233 -j ACCEPT
```

**Router:**
1. Log into your router admin panel
2. Find "Port Forwarding" settings
3. Forward TCP port 8233 to your computer's local IP
4. Save and restart router

### Monitoring

Check your node status anytime:

```bash
# Zebra status
zebrad status

# Check if serving peers
netstat -an | grep 8233

# View Zebra logs
tail -f ~/.zebra/debug.log
```

## Troubleshooting

### "Zebra not found" error

```bash
# Add Zebra to PATH (Linux/Mac)
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Or specify location manually
./scripts/generate_proof.sh --zebra-dir ~/.zebra
```

### "Proof generation failed"

Common causes:
1. Zebra not fully synced (check `zebrad status`)
2. Zebra not running (start with `zebrad start`)
3. Insufficient disk space (need 100+ GB)

### "Sync is stuck"

```bash
# Restart Zebra
pkill zebrad
zebrad start

# Check logs for errors
tail -100 ~/.zebra/debug.log
```

### Need Help?

- **Discord:** [discord.gg/depinzcash](https://discord.gg/depinzcash) ← Fastest
- **GitHub:** [github.com/depinzcash/DePINZcash/issues](https://github.com/depinzcash/DePINZcash/issues)
- **Email:** support@depinzcash.io

## Expected Timeline

| Step | Duration | Can Run in Background? |
|------|----------|------------------------|
| Install Zebra | 15 min | No |
| Sync Zebra | 6-48 hours | Yes ✓ |
| Install Prover | 5 min | No |
| Generate Proof | 2 min | No |
| Submit Proof | 1 min | No |
| **Total active time** | **~25 minutes** | - |

## Estimated Earnings

If you start today and run 24/7:

| Timeframe | Estimated Earnings |
|-----------|-------------------|
| Week 1 | 0.75 ZEC (~$22) |
| Month 1 | 1.58 ZEC (~$47) |
| Month 3 | 3.24 ZEC (~$97) |
| Year 1 | 13.64 ZEC (~$409) |

*At $30/ZEC. Actual earnings vary based on uptime and network participation.*

## Commands Cheat Sheet

```bash
# Zebra
zebrad start          # Start Zebra node
zebrad status         # Check sync progress
zebrad --version      # Check version
pkill zebrad          # Stop Zebra

# DePINZcash
./scripts/setup.sh              # Initial setup
./scripts/generate_proof.sh     # Generate proof
./scripts/generate_proof.sh -v  # Verbose mode

# Monitoring
tail -f ~/.zebra/debug.log      # Watch Zebra logs
du -sh ~/.zebra                 # Check storage usage
netstat -an | grep 8233         # Check peer connections
```

## Configuration

Edit your config anytime:

```bash
nano config/config.json
```

Update wallet addresses, node ID, or API settings.

## Useful Links

- **Zebra Docs:** https://zebra.zfnd.org/
- **Zcash Explorer:** https://explorer.zcha.in/
- **Rewards Guide:** [docs/REWARDS.md](docs/REWARDS.md)
- **Full Tech Spec:** [docs/TECHNICAL_SPEC.md](docs/TECHNICAL_SPEC.md)
- **FAQ:** [docs/FAQ.md](docs/FAQ.md)

---

## Ready to Start?

```bash
# 1. Install Zebra
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install --locked zebrad

# 2. Start syncing
zebrad start

# 3. Install prover
git clone https://github.com/depinzcash/DePINZcash
cd DePINZcash
./scripts/setup.sh

# 4. Wait for sync, then generate proof
./scripts/generate_proof.sh

# 5. Submit at https://depinzcash.io/submit
```

**Welcome to DePINZcash! Start earning while strengthening Zcash privacy infrastructure.** 🦓⚡

Questions? Join our Discord: https://discord.gg/depinzcash
