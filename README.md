# DePINZcash

**Decentralized Physical Infrastructure Network for Zcash**

Incentive layer for Zcash nodes. Earn rewards for strengthening privacy infrastructure through zero-knowledge verified node operation.

## What is DePINZcash?

DePINZcash rewards users for running Zcash full nodes by providing cryptographic proof of their contribution to the network. Using zero-knowledge proofs (Halo 2), users can prove they're running a synced Zebra node without revealing sensitive information.

## How It Works

1. **Run Zebra**: Download and sync the official Zebra full node client
2. **Generate Proofs**: Use our proof generator to create ZK proofs of your node operation
3. **Submit & Earn**: Submit proofs to our platform and receive rewards in SOL or ZEC

## Features

- ✅ Zero-knowledge proof verification using Halo 2
- ✅ No modifications to official Zebra client required
- ✅ Privacy-preserving node metrics
- ✅ Automatic checkpoint-based proof generation
- ✅ Rewards for sync completion and uptime
- ✅ Flexible payouts in SOL (Solana) or ZEC (Zcash)

## Quick Start

### Prerequisites

- [Zebra](https://github.com/ZcashFoundation/zebra) installed and synced
- Rust 1.70+ (for building the proof generator)
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/ZcashDePIN/DePINZcash
cd DePINZcash

# Run setup
./scripts/setup.sh

# Configure your wallet(s)
# Provide your Solana wallet and/or Zcash address for receiving rewards
```

### Generate Your First Proof

```bash
# Generate a proof (run this after your Zebra node is synced)
./scripts/generate_proof.sh

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

## Rewards System

Node operators earn rewards for verifiably contributing to the Zcash network infrastructure. The reward pool is funded by protocol fees collected in a secure vault, ensuring sustainable and transparent payouts.

### Payment Options

Choose your preferred payment method:
- **SOL (Solana)** - Fast, low-fee transactions (recommended)
- **ZEC (Zcash)** - Privacy-focused payments

### How Rewards Work

- **Initial Sync**: Bonus rewards for completing blockchain synchronization
- **Uptime**: Continuous rewards for keeping your node online and synced
- **Network Participation**: Additional rewards for serving peers and strengthening the network
- **Fee Vault**: Protocol fees are collected in a vault and distributed to verified node operators

See [REWARDS.md](docs/REWARDS.md) for detailed information about the reward structure.

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
- Forum: [discussions](https://github.com/ZcashDePIN/DePINZcash/discussions)

## License

MIT License - see [LICENSE](LICENSE) for details

## Acknowledgments

- [Zcash Foundation](https://www.zfnd.org/) for Zebra
- [Electric Coin Company](https://electriccoin.co/) for Zcash
- Halo 2 proving system

---

**DePINZcash** - Strengthening privacy infrastructure, one node at a time. 🦓⚡
