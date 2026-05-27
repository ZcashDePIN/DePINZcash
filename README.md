# DePINZcash

**Decentralized Physical Infrastructure Network for Zcash**

Incentive layer for Zcash nodes. Earn **$ZePIN** for running a Zebra full node or a lightwalletd server — the backend verifies your node against a trusted-RPC quorum and distributes rewards on Solana.

- **Site**: [zcashdepin.com](https://zcashdepin.com)
- **API**: [api.zcashdepin.com](https://api.zcashdepin.com)
- **Explorer**: [zcashdepin.com/explorer](https://zcashdepin.com/explorer) — every block hash links to Blockchair for independent verification
- **X / Twitter**: [@DePINZcash](https://x.com/DePINZcash)

> **$ZePIN on Solana, for now.** Until [NU7](https://zips.z.cash/protocol/nu7) and [ZIP-227](https://zips.z.cash/zip-0227) ship custom assets on Zcash natively, rewards settle as $ZePIN on Solana mainnet-beta. Once native Zcash custom assets land, the protocol migrates without changing the operator flow.

---

## Architecture

```
   ┌──────────────────┐   sign + POST     ┌─────────────────────┐  RPC quorum  ┌──────────────────┐
   │   depinzcash-    │ ─────────────────▶│  depinzcash-server  │ ───────────▶ │  Trusted Zcash   │
   │   relay (CLI)    │                   │   (Rust / Axum)     │              │  full nodes      │
   └────────▲─────────┘                   └─────────┬───────────┘              └──────────────────┘
            │                              ┌────────┘
            │ reads Zebra metrics          │ polls operator's RPC (Exposed RPC mode)
            ▼                              ▼
   ┌──────────────────┐           ┌─────────────────────┐
   │ Local Zebra full │           │  zepin-claim         │
   │ node             │           │  (Anchor / Solana)   │
   └──────────────────┘           └─────────────────────┘
```

Three components, one repo:

- **[server/](server/)** — Rust / Axum backend. Verifies proofs against a trusted-RPC quorum, runs the points/uptime/exposed-RPC scheduler, builds Merkle snapshots for $ZePIN claim distribution. Deployed on Fly.io.
- **[prover/](prover/)** — `depinzcash-relay` CLI: operator-side binary that signs node-state submissions with a Solana keypair and posts them to the server. Supports `keygen`, `register`, `submit`, and `watch` subcommands.
- **[web/](web/)** — React + Vite + Tailwind frontend. Deployed on Vercel.
- **[programs/zepin-claim/](programs/zepin-claim/)** — Anchor scaffold for the $ZePIN Merkle-distributor claim program on Solana. Matches `server/src/merkle.rs` byte-for-byte (SHA-256 leaf hashing, sorted-pair internal nodes).
- **[docs/](docs/)** — Operator guides including [Exposed RPC setup](docs/EXPOSED_RPC.md).

---

## Node types

| Kind | Reward tier | Disk | RAM | When to choose |
|---|---|---|---|---|
| `zebra-full` | Higher (10) | ~120 GB | 4–8 GB | You already run a full node or want the highest payout |
| `lightwalletd` | Lower (6) | ~30 GB (+ backing Zebra) | 1–2 GB | **Recommended for newcomers** — easier setup |

Setup guides: [/run-node](https://zcashdepin.com/run-node) (Zebra) · [/run-lightwalletd](https://zcashdepin.com/run-lightwalletd)

---

## Verification modes

| Mode | Status | Operator install | How it works |
|---|---|---|---|
| **Relay CLI** | ✅ Active | `depinzcash-relay` binary | Operator-initiated: relay reads Zebra's tip every 5 min, signs with the Solana keypair, POSTs to the server. Quorum cross-checks the block hash. |
| **Exposed RPC** | ✅ Active | Nothing from us | Server-initiated: operator registers their public RPC URL, server polls `getblockcount` + `getblockhash` every 5 min. Zero binary install. [Setup guide →](docs/EXPOSED_RPC.md) |

---

## Registration (CLI only)

Node registration is done from the terminal — no browser wallet needed:

```bash
# build the relay (one-time)
git clone https://github.com/ZcashDePIN/DePINZcash
cd DePINZcash/prover
cargo build --release --bin depinzcash-relay
sudo cp ./target/release/depinzcash-relay /usr/local/bin/

# generate a Solana keypair
mkdir -p ~/.depinzcash
depinzcash-relay keygen --out ~/.depinzcash/solana-keypair.json

# register (relay mode)
depinzcash-relay register \
    --api https://api.zcashdepin.com \
    --keypair ~/.depinzcash/solana-keypair.json \
    --kind zebra-full \
    --label my-node

# or register with exposed RPC (no relay needed after this)
depinzcash-relay register \
    --api https://api.zcashdepin.com \
    --keypair ~/.depinzcash/solana-keypair.json \
    --kind zebra-full \
    --label my-node \
    --rpc-endpoint https://zebra.yourdomain.com

# start submitting proofs (relay mode)
depinzcash-relay watch \
    --state ~/.depinzcash/relay-state.json \
    --node-rpc http://127.0.0.1:8232
```

---

## Rewards

Points accrue per accepted proof and per uptime tick:

```
points = tier * (1 + freshness)  +  min(uptime_hours, 24)  +  min(peers/4, 3)
where: tier        = 10 (zebra-full) | 6 (lightwalletd)
       freshness   = max(0, 5 - height_drift_from_trusted_tip)
```

Weekly Merkle snapshots (`SNAPSHOT_INTERVAL`) hash `(wallet, points)` pairs into a sorted-pair SHA-256 tree. The Solana claim program verifies proofs against the published root. Operators fetch their claim:

```
GET /api/wallet/<solana-pubkey>/claim/latest
```

---

## Anti-bot protections

- Ed25519 Solana signatures on every registration + proof submission
- Per-wallet nonce table (single-use, prevents replay)
- `MAX_NODES_PER_WALLET` cap (default 5) — blocks label-spam farming
- `MIN_REAL_HEIGHT` filter (default 3,000,000) — bots submitting fake heights below mainnet tip are invisible to all public stats
- Per-IP rate limiting via `Fly-Client-IP` header (not TCP peer)
- Localhost / private-IP RPC endpoints rejected at registration
- Kill-switches: `REGISTRATION_ENABLED`, `PROOF_SUBMISSION_ENABLED`, `SCHEDULER_ENABLED` — flip via `fly secrets set`
- Admin cleanup endpoint: batched purge of fake-height nodes + per-wallet cap enforcement

---

## API surface

| Method | Path | Notes |
|--------|------|-------|
| GET | `/healthz`, `/readyz` | Liveness + readiness |
| GET | `/api/info` | Version, network, features, $ZePIN mint |
| POST | `/api/nodes/register` | Signed registration → `node_id` + `auth_token` |
| GET | `/api/nodes` | Explorer: active nodes (last 1h, height ≥ 3M) |
| GET | `/api/nodes/:id` | Single node detail |
| GET | `/api/nodes/:id/proofs` | Per-node proof history |
| GET | `/api/nodes/:id/series` | Daily points buckets (14d bar chart) |
| GET | `/api/wallet/:wallet/nodes` | Nodes owned by wallet |
| GET | `/api/wallet/:wallet/stats` | Aggregate points + uptime |
| GET | `/api/wallet/:wallet/proofs` | Recent proofs |
| GET | `/api/wallet/:wallet/claim/latest` | Latest Merkle claim payload |
| POST | `/api/proofs/submit` | Signed proof submission |
| GET | `/api/proofs/recent` | Global proof feed (filterable: `?verdict=accepted&wallet=...`) |
| POST | `/api/challenges/request` | Random-depth block-hash challenge |
| POST | `/api/challenges/submit` | Challenge answer |
| GET | `/api/stats/network` | Network-wide totals (cached 5 min) |
| GET | `/api/stats/leaderboard` | Top wallets by points (cached 5 min) |
| GET | `/api/snapshots/latest` | Latest published snapshot |
| POST | `/api/admin/snapshot/publish` | Force-publish (`x-admin-key`) |
| POST | `/api/admin/nodes/:id/purge` | Delete node + CASCADE (`x-admin-key`) |
| POST | `/api/admin/nodes/:id/suspend` | Suspend node (`x-admin-key`) |
| POST | `/api/admin/cleanup` | Batched bot purge — dry-run default (`x-admin-key`, `?confirm=true`) |

---

## Configuration

Server reads env vars. Key knobs:

| Var | Default | Purpose |
|-----|---------|---------|
| `BIND_ADDR` | `0.0.0.0:3000` | HTTP listener |
| `DATABASE_URL` | `sqlite://depinzcash.sqlite?mode=rwc` | SQLite DSN |
| `ZCASH_NETWORK` | `mainnet` | `mainnet` or `testnet` |
| `SOLANA_CLUSTER` | `devnet` (prod: `mainnet-beta`) | Surfaced to clients |
| `TRUSTED_RPCS` | (empty) | Comma-sep Zcash JSON-RPC quorum |
| `ADMIN_API_KEY` | (empty) | Required for `/api/admin/*` |
| `MAX_HEIGHT_DRIFT` | `8` | Reject proofs diverging by more |
| `MAX_CLOCK_SKEW` | `15m` | Timestamp window |
| `SNAPSHOT_INTERVAL` | `7d` | Reward snapshot cadence |
| `EXPOSED_RPC_POLL_INTERVAL` | `off` (prod: `5m`) | Exposed RPC polling frequency |
| `MAX_NODES_PER_WALLET` | `5` | Per-wallet registration cap |
| `MIN_REAL_HEIGHT` | `3000000` | Fake-height filter for public stats |
| `RATE_LIMIT_RPS` | `2` | Per-IP requests/second |
| `RATE_LIMIT_BURST` | `10` | Per-IP burst allowance |
| `REGISTRATION_ENABLED` | `true` | Kill-switch for new registrations |
| `PROOF_SUBMISSION_ENABLED` | `true` | Kill-switch for proof submissions |

---

## Tests

```bash
cd server && cargo test          # 200+ tests in ~0.3s
cd prover && cargo test          # relay unit tests
cd server && cargo kani           # 15 formal-verification harnesses (optional, needs kani-verifier)
```

**200+ server tests across 11 files:**

| Suite | Tests | What it covers |
|---|---|---|
| Unit + proptest | ~100 | Merkle tree, auth, RPC, config, points formula, normalize_hash, `is_unreachable_host`, `FlyClientIpKeyExtractor`. 6 proptest properties (256 random cases each). |
| `e2e_register_and_proof` | 6 | Full router round-trip: register → submit → leaderboard → snapshot → claim |
| `adversarial_register` | 16 | Bad-input rejections: bad sig, replayed nonce, stale timestamp, bad RPC scheme, per-wallet cap |
| `adversarial_proof` | 14 | Wrong wallet, replayed nonce, empty/oversized hash, monotonic-height guard |
| `exposed_rpc` | 5 | Mock zcashd: accept/reject/dedupe/drift/no-endpoint |
| `exposed_rpc_live` | 3 | Real Zcash JSON-RPC (ignored by default, opt-in via `LIVE_ZEBRA_RPC` env) |
| `store_conformance` | 23 | SQLite CRUD, uniqueness, snapshots, nonce single-use |
| `rpc_quorum` | 11 | Mock RPC servers: majority, no-quorum, all-failing, type mismatch |
| `health_info_cors` | 8 | `/healthz`, `/readyz`, `/api/info`, CORS |
| `snapshots` | 8 | Merkle publish + claim lifecycle |
| `challenges_http` | 7 | Challenge request/submit/expiry |
| `concurrency` | 5 | Race-safe proof insertion |

**15 Kani formal-verification harnesses** prove (for all bounded inputs, not sampled):
- Points engine: upper bound, monotonic in uptime/peers, anti-monotonic in drift, tier-ordering, zero-tier ceiling
- Merkle: every leaf verifies, tampered-byte rejects, truncated proof rejects, deterministic builds, commutative pair-hashing
- Auth: nonce length invariant, registration message determinism, kind-changes-output

---

## License

MIT — see [LICENSE](LICENSE).
