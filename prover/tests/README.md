# DePINZcash Tests

This directory contains tests for the proof generator.

## Running Tests

### Run all tests
```bash
cd prover
cargo test
```

### Run specific test file
```bash
cargo test --test integration_test
```

### Run with output
```bash
cargo test -- --nocapture
```

### Run a specific test
```bash
cargo test test_proof_generation_basic
```

## Test Coverage

### Unit Tests (in source files)
- `config.rs` - Configuration validation
- `proof_gen.rs` - Reward calculations
- `zebra_reader.rs` - State reading logic

### Integration Tests
- `integration_test.rs` - End-to-end proof generation
- `test_helpers.rs` - Mock data generators

## What's Tested

✅ Proof generation workflow
✅ Reward calculation logic
✅ Config validation
✅ Proof serialization/deserialization
✅ Different sync percentages
✅ Peer multiplier effects
✅ File I/O operations

## Adding New Tests

Create test functions with `#[test]` attribute:

```rust
#[test]
fn test_my_feature() {
    // Your test code
    assert_eq!(1 + 1, 2);
}
```

Use the helpers in `test_helpers.rs` for mock data:

```rust
use test_helpers::*;

#[test]
fn my_test() {
    let metrics = mock_node_metrics();
    // Test with metrics
}
```

## CI/CD

Tests run automatically on:
- Every commit
- Pull requests
- Before releases

All tests must pass before merging.
