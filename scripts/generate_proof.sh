#!/bin/bash

set -e

echo "🦓 DePINZcash Proof Generator"
echo "=============================="
echo ""

# Check if proof generator is built
if [ ! -f "prover/target/release/depinzcash-prover" ] && [ ! -f "prover/target/release/depinzcash-prover.exe" ]; then
    echo "❌ Proof generator not built."
    echo "Please run ./scripts/setup.sh first"
    exit 1
fi

# Check if config exists
if [ ! -f "config/config.json" ]; then
    echo "❌ Configuration not found."
    echo "Please run ./scripts/setup.sh first"
    exit 1
fi

# Ensure proofs directory exists
mkdir -p proofs

# Run the proof generator
echo "Generating proof..."
echo ""

if [ -f "prover/target/release/depinzcash-prover.exe" ]; then
    # Windows
    ./prover/target/release/depinzcash-prover.exe "$@"
else
    # Linux/Mac
    ./prover/target/release/depinzcash-prover "$@"
fi

# Check for the most recent proof
latest_proof=$(ls -t proofs/proof_*.json 2>/dev/null | head -n1)

if [ -n "$latest_proof" ]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ Success! Proof generated:"
    echo "   $latest_proof"
    echo ""
    echo "📊 Estimated reward:"

    # Parse and display reward (if jq is available)
    if command -v jq &> /dev/null; then
        block_height=$(jq -r '.metrics.block_height' "$latest_proof")
        sync_pct=$(jq -r '.metrics.sync_percentage' "$latest_proof")
        uptime=$(jq -r '.metrics.uptime_hours' "$latest_proof")

        echo "   Block height: $block_height"
        echo "   Sync: ${sync_pct}%"
        echo "   Uptime: ${uptime} hours"
    fi

    echo ""
    echo "📤 Next step:"
    echo "   Upload this file to: https://depinzcash.io/submit"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
fi
