# DePINZcash

**Decentralized Physical Infrastructure Network for Zcash**

Incentive layer for Zcash nodes. Earn rewards for strengthening privacy infrastructure through zero-knowledge verified node operation.

## What is DePINZcash?

DePINZcash rewards users for running Zcash full nodes by providing cryptographic proof of their contribution to the network. Using zero-knowledge proofs (Halo 2), users can prove they're running a synced Zebra node without revealing sensitive information.

## How It Works

1. **Run Zebra**: Download and sync the official Zebra full node client
2. **Generate Proofs**: Use our proof generator to create ZK proofs of your node operation
3. **Submit & Earn**: Submit proofs to our platform and receive ZEC/SOL rewards

## Features

- ✅ Zero-knowledge proof verification using Halo 2
- ✅ No modifications to official Zebra client required
- ✅ Privacy-preserving node metrics
- ✅ Automatic checkpoint-based proof generation
- ✅ Rewards for sync completion and uptime

## Quick Start

### Prerequisites

- [Zebra](https://github.com/ZcashFoundation/zebra) installed and synced
- Rust 1.70+ (for building the proof generator)
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/DePINZcash
cd DePINZcash

# Run setup
./setup.sh

# Configure your wallets
# Follow the prompts to enter your Solana and Zcash addresses
```

### Generate Your First Proof

```bash
# Generate a proof (run this after your Zebra node is synced)
./generate_proof.sh

# The proof will be saved to ./proofs/proof_[timestamp].json
# Upload this file to https://depinzcash.io/submit
```

## Project Structure

```
DePINZcash/
├── prover/                 # Rust proof generator
│   ├── src/
│   │   ├── main.rs
│   │   ├── zebra_reader.rs
│   │   ├── proof_gen.rs
│   │   └── config.rs
│   ├── Cargo.toml
│   └── README.md
├── scripts/                # Shell scripts
│   ├── generate_proof.sh
│   ├── setup.sh
│   └── verify_zebra.sh
├── config/                 # Configuration templates
│   └── config.example.json
├── proofs/                 # Generated proofs stored here
├── docs/                   # Documentation
│   ├── TECHNICAL_SPEC.md
│   ├── REWARDS.md
│   └── FAQ.md
└── README.md
```

## Rewards

### Initial Sync Bonus
- 0.5 ZEC for 100% sync
- 0.375 ZEC for 90-99% sync
- 0.25 ZEC for 75-89% sync

### Uptime Rewards
- 0.001 ZEC/hour base rate
- 1.5x multiplier if serving peers
- 1.0x multiplier if not serving

See [REWARDS.md](docs/REWARDS.md) for detailed reward calculations.

## What Gets Proven?

### Public Inputs (Revealed)
- Block height reached
- Timestamp of checkpoint
- Proof submission ID

### Private Inputs (Hidden)
- Node transaction history
- Connection details
- Full sync logs

**The ZK proof attests**: *"I have synced a Zebra node to block height X at time T"*

## Security

- ✅ Binary hash verification ensures official Zebra software
- ✅ ZK proofs prevent information leakage
- ✅ Replay protection prevents proof reuse
- ✅ State verification against live Zcash network
- ✅ Rate limiting prevents abuse

## Roadmap

### Phase 1 (Current)
- [x] Technical specification
- [ ] Proof generator implementation
- [ ] Manual proof submission website
- [ ] Beta testing with 10-20 node operators

### Phase 2
- [ ] Automatic proof submission
- [ ] lightwalletd server rewards
- [ ] Automated reward distribution
- [ ] Mobile monitoring app

### Phase 3
- [ ] Mining integration
- [ ] Cross-chain verification
- [ ] Decentralized verifier network

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Community

- Twitter: [@DePINZcash](https://twitter.com/DePINZcash)
- Discord: [Join our server](https://discord.gg/depinzcash)
- Forum: [discussions](https://github.com/your-org/DePINZcash/discussions)

## License

MIT License - see [LICENSE](LICENSE) for details

## Acknowledgments

- [Zcash Foundation](https://www.zfnd.org/) for Zebra
- [Electric Coin Company](https://electriccoin.co/) for Zcash
- Halo 2 proving system

---

**DePINZcash** - Strengthening privacy infrastructure, one node at a time. 🦓⚡
