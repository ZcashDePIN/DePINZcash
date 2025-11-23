# DePINZcash FAQ

## General Questions

### What is DePINZcash?

DePINZcash is a DePIN (Decentralized Physical Infrastructure Network) protocol that rewards people for running Zcash full nodes. You earn cryptocurrency (ZEC or SOL) for helping strengthen the Zcash privacy network.

### Why should I run a Zcash node?

**Earn rewards** while supporting privacy infrastructure. Plus:
- Help decentralize Zcash
- Improve network security
- Support financial privacy
- Learn about blockchain technology

### Do I need to modify Zebra?

**No!** You run the official, unmodified Zebra client. Our proof generator is a separate tool that just reads Zebra's data.

### Is this official?

DePINZcash is an independent project, not officially affiliated with Zcash Foundation or Electric Coin Company. However, we're building on their open-source software and contributing to the Zcash ecosystem.

## Getting Started

### What do I need to run a node?

**Minimum requirements:**
- Computer (PC, Mac, or Linux)
- 100+ GB free disk space
- 8+ GB RAM (16 GB recommended)
- Stable internet connection
- Electricity (node runs 24/7)

**Software:**
- Zebra full node client
- Rust (to build the prover)
- Git (to clone our repo)

### How long does initial sync take?

**Depends on your hardware:**
- Fast SSD + good internet: 6-12 hours
- Average hardware: 1-2 days
- Slow hardware: 3-5 days

Zcash blockchain is currently ~50 GB and growing.

### Can I run this on a Raspberry Pi?

**Not recommended** for mainnet. Raspberry Pi 4 with 8GB RAM might work for testnet, but sync will be very slow. Use a desktop or server for best results.

### How much does it cost to run?

**Estimated monthly costs:**
- Electricity: $5-15 (depends on location)
- Internet: Usually included in home plan
- Hardware: One-time $300-1000 (if buying new)

Most people use existing computers, so main cost is electricity.

## Technical Questions

### How do zero-knowledge proofs work?

ZK proofs let you prove something is true without revealing the underlying data.

**In DePINZcash:**
- You prove: "I synced to block X at time T"
- Without revealing: Your IP, transaction data, or other private info

We use **Halo 2**, the same ZK system used by Zcash itself.

### What data does the proof contain?

**Public (visible to everyone):**
- Block height you've reached
- Timestamp of proof generation
- Network (mainnet/testnet)
- Your wallet addresses (for payment)

**Private (proven but not revealed):**
- Zebra binary hash
- Exact uptime
- Peer connection details
- Full sync history

### Can I fake a proof?

**No.** Zero-knowledge proofs are cryptographically secure. You can't create a valid proof without actually running the node. Attempting to do so would require breaking modern cryptography.

### How do you prevent cheating?

Multiple layers:
1. **ZK proofs** - cryptographically impossible to forge
2. **Binary verification** - ensures official Zebra software
3. **State verification** - cross-checks block height with network
4. **Replay prevention** - each proof is unique and tracked
5. **Rate limiting** - max 1 proof per 24 hours

### What if Zebra updates?

Just update Zebra normally:
```bash
# Update Zebra
cargo install zebrad --locked

# Continue running as usual
zebrad start
```

Our prover will automatically detect the new version.

## Rewards & Payments

### How much can I earn?

**Approximate monthly earnings** (24/7 uptime, serving peers):
- Week 1: ~0.75 ZEC ($22.50 at $30/ZEC)
- Month 1: ~1.58 ZEC ($47.40)
- Month 3: ~1.08 ZEC/month ($32.40/month)
- Year 1: ~13.64 ZEC ($409.20)

See [REWARDS.md](REWARDS.md) for detailed breakdown.

### When do I get paid?

**Phase 1 (current):** Weekly batch payments every Monday

**Phase 2 (future):** Instant automated payouts

### What if ZEC price drops?

Rewards are denominated in ZEC, so you always earn the same amount of ZEC. Dollar value fluctuates with market price.

We may adjust reward rates if ZEC price changes significantly.

### Can I get paid in other crypto?

Currently:
- ✅ ZEC (Zcash)
- ✅ SOL (Solana)

Future options:
- BTC (Bitcoin)
- ETH (Ethereum)
- Stablecoins (USDC, USDT)

### Is there a minimum payout?

**Current minimum:** 0.1 ZEC

Rewards accumulate until you reach this threshold. For daily runners, you'll hit this in ~2 weeks.

### What about transaction fees?

**We cover all transaction fees.** You receive the full reward amount with no deductions.

## Privacy & Security

### Does running a node expose my IP?

Yes, like any P2P network participant, your IP is visible to peers. If privacy is a concern:
- Use a VPS (cloud server)
- Run through VPN
- Use Tor (slower, not recommended for full nodes)

### Can you see my Zcash transactions?

**No.** The proof only reveals:
- Block height
- Timestamp
- Network type

Your transaction history remains private.

### Is my wallet address exposed?

Yes, we need your wallet address to send rewards. This is similar to mining pool payouts.

If you use a **Zcash shielded address (z-addr)**, the payment itself is private.

### What permissions does the prover need?

**Read-only access** to:
- Zebra's state database
- Zebra binary (for hash verification)

**No write access.** The prover cannot modify Zebra or your system.

## Troubleshooting

### My node won't sync

**Common fixes:**
1. Check internet connection
2. Ensure ports are open (8233 for mainnet)
3. Check disk space (need 100+ GB)
4. Restart Zebra: `zebrad start`
5. Check logs: `~/.zebra/debug.log`

### Proof generation fails

**Possible causes:**
1. Zebra not fully synced yet
2. Zebra not running
3. Incorrect configuration
4. Insufficient permissions

**Fix:**
```bash
# Check Zebra status
ps aux | grep zebrad

# Verify config
cat config/config.json

# Run with verbose logging
./scripts/generate_proof.sh --verbose
```

### "Zebra not found" error

The prover can't locate Zebra. Try:

```bash
# Specify Zebra directory manually
./scripts/generate_proof.sh --zebra-dir ~/.zebra
```

Or add `zebrad` to your PATH.

### Proof rejected by website

**Reasons:**
1. Invalid proof format
2. Block height mismatch with network
3. Duplicate submission (already claimed)
4. Rate limit exceeded (wait 24 hours)

Check your email for specific rejection reason.

## Node Operation

### Should I run mainnet or testnet?

**Mainnet** for real rewards. Use testnet only for:
- Testing setup
- Learning
- Development

Testnet rewards are minimal or zero.

### How much bandwidth does it use?

**Initial sync:** 50-100 GB download

**Ongoing:**
- Inbound: ~1-5 GB/day (serving peers)
- Outbound: ~500 MB/day (staying synced)

**Monthly total:** ~30-150 GB depending on peer activity

### Can I pause my node?

Yes, you can stop and restart anytime. Uptime rewards pause when offline, but there's no penalty. Your sync progress is saved.

### Do I need to keep my computer on 24/7?

**For maximum rewards, yes.** But you can run part-time:
- Nights only
- Weekends
- Whenever convenient

You'll earn proportionally to actual uptime.

### Can I run on a VPS/cloud server?

**Absolutely!** Many users run on:
- AWS EC2
- Google Cloud
- DigitalOcean
- Linode
- Hetzner

**Recommended specs:**
- 2+ vCPUs
- 8+ GB RAM
- 150+ GB SSD
- ~$20-40/month

## Advanced

### Can I run multiple nodes?

Yes! Each needs:
- Different wallet address (or we'll treat them as one node)
- Separate hardware/VPS
- Independent Zebra installation

This multiplies your earnings linearly.

### What about lightwalletd?

**Phase 2 feature** - not yet available. When ready, you'll earn extra for hosting lightwalletd servers that serve light clients.

### Can I modify the reward calculation?

No, rewards are determined server-side during verification. However:
- Code is open source (you can audit)
- Future: DAO governance for reward adjustments

### How is uptime calculated?

Estimated from:
1. Time since last proof submission
2. Zebra state database modification time
3. System uptime (if available)

It's approximate, not exact to the second.

### What's the proof generation algorithm?

Simplified flow:
```
1. Read Zebra state (RocksDB)
2. Extract metrics (block height, uptime, etc.)
3. Create circuit inputs (public + private)
4. Generate Halo 2 proof
5. Sign proof (Ed25519)
6. Output JSON file
```

See [TECHNICAL_SPEC.md](TECHNICAL_SPEC.md) for details.

## Community & Support

### Where can I get help?

- **Discord:** [discord.gg/depinzcash](https://discord.gg/depinzcash) - fastest
- **Twitter:** [@DePINZcash](https://twitter.com/DePINZcash)
- **GitHub Issues:** [github.com/depinzcash/DePINZcash/issues](https://github.com/depinzcash/DePINZcash/issues)
- **Email:** support@depinzcash.io

### Can I contribute to the project?

**Yes!** We're open source. Contribute via:
- Code (Rust, web frontend)
- Documentation
- Testing
- Community support
- Translations

See [CONTRIBUTING.md](../CONTRIBUTING.md).

### Is there a bug bounty?

**Coming soon.** We'll announce when ready. For now, responsible disclosure:
- Email: security@depinzcash.io
- PGP key: [link to key]

### Where's the roadmap?

See [README.md](../README.md) for current roadmap. Summary:

- **Phase 1 (now):** MVP with manual rewards
- **Phase 2:** Automation + lightwalletd
- **Phase 3:** Mining integration + advanced features

## Legal & Compliance

### Is this legal in my country?

**Generally yes**, but cryptocurrency regulations vary by jurisdiction. Running a full node is typically legal everywhere. Check local laws regarding:
- Cryptocurrency mining/earning
- Tax obligations
- KYC/AML requirements (usually not for nodes)

**We are not lawyers.** Consult local legal counsel if unsure.

### Do I need to KYC?

**Currently no.** DePINZcash doesn't collect personal information beyond wallet addresses.

This may change if regulations require it, but we'll notify users in advance.

### What about taxes?

Earning rewards may be taxable in your jurisdiction as:
- Mining income
- Self-employment income
- Capital gains (when sold)

**Keep records** of:
- All proofs submitted
- Rewards received
- Dates and amounts

DePINZcash provides CSV export for tax reporting.

### Where is DePINZcash incorporated?

**TBD.** Currently operating as an open-source project. Future: DAO structure for decentralized governance.

## Still Have Questions?

- **Quick questions:** [Discord](https://discord.gg/depinzcash)
- **Technical issues:** [GitHub Issues](https://github.com/depinzcash/DePINZcash/issues)
- **Private inquiries:** support@depinzcash.io

---

**Ready to get started?** See [README.md](../README.md) for installation instructions! 🦓⚡
