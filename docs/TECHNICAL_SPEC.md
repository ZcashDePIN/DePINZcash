# DePINZcash Technical Specification

**Version:** 1.0
**Last Updated:** November 2024

## Overview

DePINZcash is a Decentralized Physical Infrastructure Network (DePIN) protocol that incentivizes users to run Zcash full nodes by providing cryptographic proof of their contribution to the network.

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                        User's Machine                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐         ┌─────────────────────────┐      │
│  │              │         │                         │      │
│  │    Zebra     │◄────────│  DePINZcash Prover     │      │
│  │  Full Node   │  reads  │  (Rust Application)    │      │
│  │              │  state  │                         │      │
│  └──────────────┘         └─────────────────────────┘      │
│         │                           │                       │
│         │                           │ generates             │
│         │ syncs with                │                       │
│         │ network                   ▼                       │
│         │                   ┌──────────────┐                │
│         │                   │ Proof File   │                │
│         │                   │ (JSON + ZKP) │                │
│         │                   └──────────────┘                │
└─────────┼───────────────────────────┼───────────────────────┘
          │                           │
          │                           │ user uploads
          │                           │
          ▼                           ▼
   ┌────────────┐            ┌─────────────────┐
   │   Zcash    │            │  DePINZcash     │
   │  Network   │            │  Website/API    │
   └────────────┘            └─────────────────┘
                                      │
                                      │ verifies proof
                                      │
                                      ▼
                              ┌────────────────┐
                              │  Reward Queue  │
                              └────────────────┘
```

### 1. Zebra Full Node

**Official Implementation:** [ZcashFoundation/zebra](https://github.com/ZcashFoundation/zebra)

Users run the official, unmodified Zebra client to sync with the Zcash network. No modifications to Zebra are required.

**Key Features:**
- Validates and stores the Zcash blockchain
- Serves blocks to peers
- Maintains network consensus
- Stores state in RocksDB

### 2. DePINZcash Prover

**Language:** Rust
**ZK System:** Halo 2

A separate application that:
1. Reads Zebra's state database
2. Collects node metrics
3. Generates zero-knowledge proofs
4. Outputs proof files for submission

**Binary Location:** `prover/target/release/depinzcash-prover`

### 3. Proof Submission Platform

**Components:**
- Web frontend for proof upload
- API for proof verification
- Database for tracking submissions
- Reward distribution system

## Zero-Knowledge Proof System

### Proof Statement

The ZK proof attests to:

> **"I have synced a Zebra node to block height X at time T"**

### Halo 2 Implementation

**Why Halo 2?**
- ✅ No trusted setup required
- ✅ Recursive proof composition
- ✅ Same system used by Zcash itself
- ✅ Excellent security properties
- ✅ Ecosystem alignment

### Public Inputs (Revealed)

These values are included in the proof and visible to verifiers:

```rust
pub struct PublicInputs {
    block_height: u64,      // Current blockchain height
    timestamp: i64,         // Unix timestamp of proof generation
    network: String,        // "mainnet" or "testnet"
}
```

### Private Inputs (Hidden)

These values are used in proof generation but NOT revealed:

```rust
struct PrivateInputs {
    zebra_binary_hash: String,  // Proves official software
    uptime_hours: f64,          // Node operation duration
    peer_count: u32,            // Peers currently connected
    blocks_served: u64,         // Approximate blocks served to network
    full_sync_logs: Vec<u8>,    // Complete sync history
}
```

### Circuit Design

The proof circuit verifies:

1. **Binary Authenticity**: Hash of Zebra binary matches known official releases
2. **State Validity**: Block height is consistent with blockchain state
3. **Temporal Consistency**: Timestamp is recent and sequential
4. **Service Metrics**: Node has been serving the network

**Pseudocode:**
```
Circuit ProofOfNode {
    // Public inputs
    public block_height: u64
    public timestamp: i64

    // Private inputs
    private binary_hash: Hash
    private uptime: Duration
    private state_proof: MerkleProof

    // Constraints
    assert binary_hash in OFFICIAL_RELEASES
    assert state_proof.verify(block_height)
    assert timestamp >= previous_proof.timestamp
    assert uptime >= MIN_UPTIME

    // Output: valid proof
}
```

## Metrics Collection

### Reading Zebra State

Zebra stores its state in a RocksDB database located at:
- **Linux/Mac:** `~/.zebra/state/`
- **Windows:** `%APPDATA%\Zebra\state\`

**Key Data Structures:**

```rust
// Zebra's state database keys (simplified)
enum StateKey {
    TipHeight,              // Current chain tip
    Block(Height),          // Block data by height
    Transaction(TxHash),    // Transaction data
    Sprout/Sapling/Orchard, // Shielded pools
}
```

### Collected Metrics

```rust
pub struct NodeMetrics {
    // Blockchain sync
    block_height: u64,
    sync_percentage: f64,  // Calculated vs network tip

    // Operation time
    uptime_hours: f64,     // Estimated from state mtime

    // Network contribution
    peer_count: u32,       // Current connections
    blocks_served: u64,    // Approximate based on logs

    // Verification
    zebra_version: String,
    zebra_binary_hash: String,
    network: String,       // mainnet/testnet

    // Metadata
    timestamp: i64,
}
```

## Proof Format

Generated proofs are saved as JSON files:

```json
{
  "version": "1.0",
  "timestamp": 1700000000,
  "node_info": {
    "zebra_version": "1.5.0",
    "zebra_binary_hash": "sha256:abc123...",
    "network": "mainnet",
    "node_id": "optional-custom-name"
  },
  "metrics": {
    "block_height": 2450123,
    "sync_percentage": 100.0,
    "uptime_hours": 720.0,
    "peer_count": 23,
    "blocks_served": 15234
  },
  "halo2_proof": "0x1a2b3c4d...",
  "public_inputs": [
    "2450123",
    "1700000000",
    "mainnet"
  ],
  "wallets": {
    "solana": "8zP3Q...",
    "zcash": "t1abc..."
  },
  "signature": "ed25519:def456..."
}
```

## Security Model

### Anti-Cheating Mechanisms

#### 1. Binary Hash Verification

```rust
fn verify_binary() -> Hash {
    let zebrad_binary = which("zebrad")?;
    let bytes = read_file(zebrad_binary)?;
    sha256(bytes)
}
```

Ensures users are running official Zebra releases, not modified versions.

#### 2. State Verification

The verification server cross-checks claimed block heights against:
- Public blockchain explorers
- Multiple trusted Zcash nodes
- Historical blockchain state

#### 3. Proof Replay Prevention

Each proof contains:
- Unique timestamp
- Reference to previous proof (chain of proofs)
- Signature from user's derived key

Database tracks all submitted proofs to prevent reuse.

#### 4. Rate Limiting

- Maximum 1 proof per wallet per 24 hours
- Prevents spam and Sybil attacks
- Enforced at API level

#### 5. Economic Security

**Future enhancement:**
- Require small stake (10 ZEC) to participate
- Slashing for proven fraud
- Makes fake proofs economically irrational

### Threat Model

| Attack | Mitigation |
|--------|------------|
| Fake proofs without running node | ZK proof cryptographically impossible to forge |
| Modified Zebra binary | Binary hash verification |
| Replay old proofs | Timestamp + proof chain tracking |
| Claim false block height | Cross-reference with network state |
| Multiple accounts (Sybil) | Rate limiting + future: staking requirement |
| Proof tampering | Ed25519 signature over proof data |

## Reward System

### Calculation Formula

```python
def calculate_reward(proof):
    # Initial sync bonus
    if proof.sync_percentage >= 100:
        sync_bonus = 0.5
    elif proof.sync_percentage >= 90:
        sync_bonus = 0.375
    elif proof.sync_percentage >= 75:
        sync_bonus = 0.25
    else:
        sync_bonus = 0.0

    # Uptime reward
    uptime_reward = proof.uptime_hours * 0.001

    # Service multiplier
    if proof.peer_count > 0:
        multiplier = 1.5
    else:
        multiplier = 1.0

    total = sync_bonus + (uptime_reward * multiplier)

    return total  # in ZEC
```

### Reward Distribution

**Phase 1 (MVP):**
- Manual review and distribution
- Weekly batch payments
- Sent to user's Zcash or Solana wallet

**Phase 2 (Automated):**
- Smart contract on Solana
- Automatic distribution after verification
- Real-time reward tracking

## API Specification

### POST /api/submit

Submit a proof for verification.

**Request:**
```http
POST /api/submit
Content-Type: multipart/form-data

{
  "proof": <file:proof_12345.json>
}
```

**Response:**
```json
{
  "success": true,
  "proof_id": "uuid-1234-5678",
  "status": "pending_verification",
  "estimated_reward": 1.58,
  "estimated_payout": "2024-12-01"
}
```

### GET /api/proof/:id

Check proof verification status.

**Response:**
```json
{
  "proof_id": "uuid-1234-5678",
  "status": "verified",
  "reward": 1.58,
  "paid": false,
  "submitted_at": "2024-11-22T12:00:00Z",
  "verified_at": "2024-11-22T12:05:00Z"
}
```

### GET /api/stats

Get network statistics.

**Response:**
```json
{
  "total_nodes": 152,
  "total_rewards_paid": 245.67,
  "average_block_height": 2450123,
  "active_participants": 142
}
```

## Development Phases

### Phase 1: MVP (4-6 weeks)
- [x] Technical specification
- [ ] Rust proof generator (mock Halo 2)
- [ ] Shell script wrapper
- [ ] Basic web submission portal
- [ ] Manual verification and rewards
- [ ] Beta test with 10-20 users

### Phase 2: Automation (6-8 weeks)
- [ ] Real Halo 2 proof implementation
- [ ] Automatic proof verification
- [ ] Solana smart contract for rewards
- [ ] Dashboard for node operators
- [ ] Public launch

### Phase 3: Enhancement (ongoing)
- [ ] lightwalletd server rewards
- [ ] Mobile monitoring app
- [ ] Decentralized verifier network
- [ ] Cross-chain reward options

## Performance Considerations

### Proof Generation Time

| Hardware | Expected Time |
|----------|---------------|
| Desktop (16GB RAM, 8 cores) | 1-2 minutes |
| Laptop (8GB RAM, 4 cores) | 3-5 minutes |
| Server (32GB RAM, 16 cores) | 30-60 seconds |

### Storage Requirements

- **Zebra state:** 50-100 GB (mainnet)
- **Prover binary:** ~10 MB
- **Proof files:** ~100 KB each
- **Logs:** ~1 MB per day

### Network Usage

- Zebra sync: 50-100 GB initial download
- Ongoing: 1-5 GB per day (serving peers)
- Proof submission: ~100 KB per proof

## Dependencies

### Rust Crates

```toml
halo2_proofs = "0.3"      # Zero-knowledge proofs
rocksdb = "0.21"          # Read Zebra state
serde = "1.0"             # Serialization
ed25519-dalek = "2.0"     # Signatures
reqwest = "0.11"          # HTTP client
tokio = "1.0"             # Async runtime
```

### External Services

- **Zcash Network:** Blockchain data source
- **Block Explorers:** State verification
- **Solana:** Reward distribution (future)

## Testing Strategy

### Unit Tests

```bash
cd prover
cargo test
```

### Integration Tests

```bash
# Test with local testnet Zebra node
./scripts/test_local.sh
```

### End-to-End Tests

1. Run Zebra testnet node
2. Generate proof
3. Submit to staging API
4. Verify reward calculation

## Deployment

### User Installation

```bash
git clone https://github.com/depinzcash/DePINZcash
cd DePINZcash
./scripts/setup.sh
./scripts/generate_proof.sh
```

### Server Deployment

```bash
# Web API (Node.js/Express)
cd api
npm install
npm start

# Verification worker (Rust)
cd verifier
cargo build --release
./target/release/verifier-worker
```

## Future Enhancements

1. **Recursive Proofs**: Aggregate multiple proofs for efficiency
2. **Privacy Pools**: Anonymize reward recipients
3. **DAO Governance**: Community control of reward parameters
4. **Multi-Chain**: Support other privacy coins (Monero, etc.)
5. **Hardware Integration**: Dedicated node hardware

## References

- [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)
- [Zebra Documentation](https://zebra.zfnd.org/)
- [Halo 2 Book](https://zcash.github.io/halo2/)
- [DePIN Alliance](https://www.depin.org/)

---

**Maintained by:** DePINZcash Team
**License:** MIT
