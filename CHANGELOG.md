# Changelog

All notable changes to DePINZcash will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Real Halo 2 proof implementation (currently using mock proofs)
- Automated proof submission
- Web dashboard for tracking rewards
- Solana smart contract for automated payouts
- lightwalletd server rewards

## [0.1.0] - 2024-11-23

### Added
- Initial project structure
- Rust proof generator with Halo 2 framework
- Zebra state reader (RocksDB integration)
- ZK proof generation (mock implementation for MVP)
- Shell scripts for setup and proof generation
- Configuration management
- Reward calculation formula
- Ed25519 proof signatures
- Comprehensive documentation:
  - Technical specification
  - Rewards guide
  - FAQ
  - Quick start guide
  - Contributing guidelines
- MIT License
- Example configuration file

### Features
- Read Zebra node metrics (block height, sync %, uptime)
- Generate zero-knowledge proof of node operation
- Save proofs as JSON files
- Calculate expected rewards
- Support for mainnet and testnet
- Binary hash verification
- Proof signing to prevent tampering

### Documentation
- Complete README with project overview
- Technical specification (10+ pages)
- Rewards breakdown with examples
- FAQ covering common questions
- Quick start guide for new users
- Contributing guidelines

### Developer Experience
- Cargo workspace setup
- Automated build scripts
- Unit tests for reward calculations
- Type-safe configuration
- Error handling with anyhow
- Structured logging with tracing

## [0.0.1] - 2024-11-22

### Added
- Project conception
- Initial repository setup
- Basic project planning

---

## Version History

### Version 0.1.0 (MVP)
**Status:** In Development
**Target:** Beta Launch

**Key Features:**
- Proof generation for Zebra nodes
- Manual proof submission
- Basic reward calculation
- Documentation and guides

**Limitations:**
- Mock ZK proofs (not production Halo 2)
- Manual reward distribution
- No automated verification
- No web dashboard

### Version 0.2.0 (Planned)
**Status:** Design Phase
**Target:** Q1 2025

**Planned Features:**
- Real Halo 2 proof implementation
- Automated proof verification API
- Basic web dashboard
- Email notifications
- Improved error handling

### Version 1.0.0 (Planned)
**Status:** Roadmap
**Target:** Q2 2025

**Planned Features:**
- Solana smart contract integration
- Automated reward distribution
- Public launch
- Mobile app for monitoring
- Advanced analytics
- Community governance

---

## Release Notes Format

Each release includes:
- **Added** - New features
- **Changed** - Changes to existing functionality
- **Deprecated** - Features to be removed
- **Removed** - Removed features
- **Fixed** - Bug fixes
- **Security** - Security improvements

## How to Upgrade

### From 0.0.x to 0.1.0

```bash
# Pull latest changes
git pull origin main

# Rebuild prover
cd prover
cargo build --release

# Update configuration (if needed)
# Check config/config.example.json for new fields
```

---

**Stay updated:** Follow [@DePINZcash](https://twitter.com/DePINZcash) for release announcements!
