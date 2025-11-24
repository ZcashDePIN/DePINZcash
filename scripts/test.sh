#!/bin/bash

set -e

echo "🧪 Running DePINZcash Tests"
echo "============================"
echo ""

cd prover

# Run all tests
echo "Running unit tests..."
cargo test --lib

echo ""
echo "Running integration tests..."
cargo test --test integration_test

echo ""
echo "Running doc tests..."
cargo test --doc

echo ""
echo "✅ All tests passed!"
