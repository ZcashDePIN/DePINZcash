# Testing DePINZcash

## Test Strategy

We've created comprehensive tests for the proof generation system.

## Test Files Created

### 1. Reward Calculation Tests (`reward_tests.rs`)
Tests the core reward logic without heavy dependencies:

✅ **Sync Bonus Tiers:**
- 100% sync → 0.5 ZEC bonus
- 90-99% sync → 0.375 ZEC bonus
- 75-89% sync → 0.25 ZEC bonus
- Below 75% → No bonus

✅ **Uptime Rewards:**
- Base rate: 0.001 ZEC per hour
- 24 hours = 0.024 ZEC
- 30 days (720h) = 0.72 ZEC

✅ **Peer Multiplier:**
- Serving peers: 1.5x multiplier
- Not serving: 1.0x (no bonus)

✅ **Real Scenarios:**
- **30 days, 100% synced, serving peers:** ~1.58 ZEC
- **7 days, 90% synced, no peers:** ~0.543 ZEC

### 2. Integration Tests (`integration_test.rs`)
Full proof generation workflow tests:

- Mock node metrics creation
- Config validation
- Proof generation end-to-end
- Proof serialization/deserialization
- Different sync percentages
- Peer multiplier effects

### 3. Workflow Tests (`proof_workflow_test.rs`)
Tests complete user workflows:

- Generate proof from metrics
- Save proof to file
- Load proof back
- Validate structure
- Test multiple scenarios

### 4. Test Helpers (`test_helpers.rs`)
Mock data generators for easy testing:

- `mock_node_metrics()` - Creates test node data
- `mock_config()` - Creates test configuration
- `mock_proof()` - Creates test proof
- Custom builders for specific scenarios

## Running Tests

### Prerequisites

**On Windows**, RocksDB tests require LLVM/Clang:
```bash
# Install LLVM from https://releases.llvm.org/
# Or use chocolatey:
choco install llvm
```

### Run Simple Tests (No RocksDB)
```bash
cd prover
cargo test --test reward_tests
```

### Run All Tests (Requires LLVM)
```bash
cd prover
cargo test
```

### Run Specific Test
```bash
cargo test test_basic_reward_math
```

### Run with Output
```bash
cargo test -- --nocapture
```

## Test Results

### Expected Output

```
running 5 tests
test reward_tests::test_basic_reward_math ... ok
test reward_tests::test_uptime_rewards ... ok
test reward_tests::test_peer_multiplier ... ok
test reward_tests::test_full_scenario ... ok
test reward_tests::test_partial_sync_scenario ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured
```

## Manual Testing

### Test Reward Calculations

You can verify rewards manually:

**Example 1: Full Setup**
- Sync: 100%
- Uptime: 720 hours (30 days)
- Peers: 15 (serving)

```
Sync bonus: 0.5 ZEC
Uptime reward: 720 × 0.001 = 0.72 ZEC
Multiplier: 1.5x (has peers)
Final uptime: 0.72 × 1.5 = 1.08 ZEC

Total: 0.5 + 1.08 = 1.58 ZEC ✓
```

**Example 2: Partial Setup**
- Sync: 85%
- Uptime: 168 hours (7 days)
- Peers: 0 (not serving)

```
Sync bonus: 0.25 ZEC (75-89% tier)
Uptime reward: 168 × 0.001 = 0.168 ZEC
Multiplier: 1.0x (no peers)
Final uptime: 0.168 × 1.0 = 0.168 ZEC

Total: 0.25 + 0.168 = 0.418 ZEC ✓
```

## Test Coverage

### What's Tested ✅

- ✅ Reward calculation formulas
- ✅ Sync percentage tiers
- ✅ Uptime tracking
- ✅ Peer multipliers
- ✅ Config validation
- ✅ Proof structure
- ✅ JSON serialization
- ✅ Multiple scenarios

### What Needs Live Testing ⚠️

- ⚠️ Actual Zebra node integration (requires running Zebra)
- ⚠️ RocksDB state reading (needs synced Zebra)
- ⚠️ Binary hash verification (needs installed Zebra)
- ⚠️ Halo 2 proof generation (mock implementation for now)

## CI/CD Integration

Tests will run automatically on:
- Every commit
- Pull requests
- Before merges
- Release builds

All basic tests must pass before code can be merged.

## Adding New Tests

Create test functions with `#[test]`:

```rust
#[test]
fn test_new_feature() {
    let result = calculate_something(10);
    assert_eq!(result, 20);
}
```

Run your new test:
```bash
cargo test test_new_feature
```

## Troubleshooting

### "libclang not found"

Install LLVM/Clang:
- **Windows**: https://releases.llvm.org/
- **Mac**: `brew install llvm`
- **Linux**: `sudo apt install clang`

### Tests pass locally but fail in CI

Make sure you're using stable Rust:
```bash
rustup default stable
```

### Want to skip slow tests

```bash
cargo test --lib  # Skip integration tests
```

## Test Philosophy

We test:
- ✅ Business logic (reward calculations)
- ✅ Data transformations
- ✅ Edge cases
- ✅ Error handling

We don't test:
- ❌ External dependencies (RocksDB, Zebra)
- ❌ Network calls
- ❌ File system operations (unless critical)

This keeps tests fast and reliable.

---

**All tests validate that the reward system works correctly and fairly!** 🧪✅
