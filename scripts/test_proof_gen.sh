#!/bin/bash

# Test script to generate sample proofs

set -e

echo "🧪 Testing Proof Generation"
echo "==========================="
echo ""

# Create test directory
mkdir -p test_output

echo "✓ Created test output directory"
echo ""
echo "📝 Test Scenarios:"
echo ""

# Test 1: Mock proof - 100% sync, serving peers
echo "Test 1: Full sync with peers..."
cat > test_output/test_proof_1.json << 'EOFPROOF'
{
  "version": "1.0",
  "timestamp": 1700000000,
  "node_info": {
    "zebra_version": "1.5.0",
    "zebra_binary_hash": "mock_hash_for_testing",
    "network": "testnet",
    "node_id": "test-node-001"
  },
  "metrics": {
    "block_height": 2450000,
    "sync_percentage": 100.0,
    "uptime_hours": 24.0,
    "peer_count": 10,
    "blocks_served": 1000
  },
  "halo2_proof": "MOCK_HALO2_PROOF_0x1a2b3c4d5e6f",
  "public_inputs": [
    "2450000",
    "1700000000",
    "testnet"
  ],
  "signature": "mock_ed25519_signature_abcdef123456"
}
EOFPROOF

echo "✓ Generated test_proof_1.json"

# Test 2: Partial sync, mainnet
echo "Test 2: Partial sync on mainnet..."
cat > test_output/test_proof_2.json << 'EOFPROOF'
{
  "version": "1.0",
  "timestamp": 1700086400,
  "node_info": {
    "zebra_version": "1.5.0",
    "zebra_binary_hash": "mock_hash_for_testing",
    "network": "mainnet",
    "node_id": "test-node-002"
  },
  "metrics": {
    "block_height": 2460000,
    "sync_percentage": 95.5,
    "uptime_hours": 168.0,
    "peer_count": 25,
    "blocks_served": 5000
  },
  "halo2_proof": "MOCK_HALO2_PROOF_0x7g8h9i0j1k2l",
  "public_inputs": [
    "2460000",
    "1700086400",
    "mainnet"
  ],
  "signature": "mock_ed25519_signature_xyz789"
}
EOFPROOF

echo "✓ Generated test_proof_2.json"

# Validate JSON structure
echo ""
echo "Validating proof structure..."

if command -v jq &> /dev/null; then
    echo ""
    echo "Proof 1 Details:"
    jq -r '.metrics | "  Block: \(.block_height) | Sync: \(.sync_percentage)% | Uptime: \(.uptime_hours)h | Peers: \(.peer_count)"' test_output/test_proof_1.json

    echo ""
    echo "Proof 2 Details:"
    jq -r '.metrics | "  Block: \(.block_height) | Sync: \(.sync_percentage)% | Uptime: \(.uptime_hours)h | Peers: \(.peer_count)"' test_output/test_proof_2.json
else
    echo "  (Install 'jq' for detailed validation)"
fi

echo ""
echo "✅ Test proofs generated in test_output/"
echo ""
echo "Files created:"
echo "  - test_output/test_proof_1.json"
echo "  - test_output/test_proof_2.json"
echo ""
echo "📤 These proofs can be submitted to your website!"
echo "   Users will enter their wallet addresses on the website form"
