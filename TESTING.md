# Testing DePINZcash

## Running tests

```bash
cd server
cargo test                    # 200+ tests, ~0.3s
cargo test -- --nocapture     # with stdout

# formal verification (optional, needs kani-verifier installed)
cargo kani

# live tests against a real Zcash RPC (optional)
LIVE_ZEBRA_RPC=https://your-zebra.example.com \
  cargo test --test exposed_rpc_live -- --include-ignored --nocapture
```

## Test surface

200+ tests across 11 integration test files + unit tests in `src/`.

### Unit tests (in src/)

- **merkle.rs** — tree construction, proof verification, leaf hashing, sorted-pair commutativity, determinism, tamper detection. 24 tests + 6 proptest properties (256 random cases each).
- **auth.rs** — signature round-trip, nonce validation, timestamp window, message field-distinguishability, sign-then-tamper rejection.
- **api/proofs.rs** — points formula (`points_from_parts`): full-credit, drift penalty, tier comparison, uptime/peers caps, `normalize_hash` idempotency + edge cases.
- **api/nodes.rs** — `is_unreachable_host` over localhost, RFC1918, link-local, broadcast, public IPs, hostnames. `validate_rpc_endpoint` scheme/shape checks.
- **api/mod.rs** — `FlyClientIpKeyExtractor`: header priority (Fly-Client-IP > X-Forwarded-For > ConnectInfo), whitespace trimming, empty-header fallback, error on missing.
- **rpc.rs** — empty quorum fails fast.
- **config.rs** — duration parsing.

### Integration tests (in tests/)

| File | Tests | Coverage |
|---|---|---|
| `e2e_register_and_proof` | 6 | Full router round-trip: register → submit → leaderboard → snapshot → claim |
| `adversarial_register` | 16 | Bad sig, replayed nonce, stale timestamp, bad RPC scheme, localhost RPC, per-wallet cap (6th node blocked) |
| `adversarial_proof` | 14 | Wrong wallet, replayed nonce, empty/oversized hash, monotonic-height guard, unknown node, suspended node |
| `exposed_rpc` | 5 | Mock zcashd servers: hash-match credits, mismatch rejects, idempotent on idle tip, drift skips, missing endpoint no-ops |
| `exposed_rpc_live` | 3 | Real Zcash RPC (ignored by default): getblockcount plausibility, getblockhash shape, end-to-end poll_one_node |
| `store_conformance` | 23 | SQLite CRUD, node uniqueness, proof dedup, snapshot lifecycle, nonce single-use, challenge expiry, stats filtering |
| `rpc_quorum` | 11 | Mock HTTP servers: 3/3 majority, 2/3 majority, no-quorum, all-failing, type mismatch, single endpoint, per-method routing |
| `health_info_cors` | 8 | `/healthz`, `/readyz`, `/api/info` fields, CORS allow/block/empty |
| `snapshots` | 8 | Empty publish fails, 404 before publish, unknown wallet 404, multi-cycle increment, claim payload shape, SPL mint passthrough |
| `challenges_http` | 7 | Challenge request/submit/expiry lifecycle |
| `concurrency` | 5 | Race-safe proof insertion (INSERT OR IGNORE), concurrent duplicate detection |

### Proptest properties

6 property-based tests that run 256 random cases each:
- Every leaf in a random tree verifies against the root
- `build_tree` is deterministic
- `hash_pair_sorted` is commutative
- Random leaf substitution breaks verification
- Appending a leaf changes the root
- `hash_leaf` is injective on distinct inputs

### Kani formal verification (15 harnesses)

Proves properties for **all** bounded inputs, not just sampled:

**Points engine (api/proofs.rs):**
- Upper bound: `points <= tier * 6 + 27`
- Uptime cap: no change once uptime ≥ 24h
- Peers cap: no change once peers ≥ 12
- Monotonic in uptime, monotonic in peers
- Anti-monotonic in drift
- Higher tier always pays ≥ lower tier
- Zero tier caps at bonuses only (≤ 27)

**Merkle (merkle.rs):**
- Any 4-leaf tree: generated proof verifies against root
- Tampered byte at any position: verification fails
- Truncated proof: verification fails
- `build_tree` deterministic for same leaves
- `hash_pair_sorted` order-independent

**Auth (auth.rs):**
- Nonce length invariant: accepted iff 16 ≤ len ≤ 128
- `registration_message` deterministic
- Distinct kinds produce distinct messages
