#!/bin/bash

set -e

echo "🦓 DePINZcash Setup"
echo "===================="
echo ""

# Check for Rust installation
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed."
    echo "Please install Rust from: https://rustup.rs/"
    exit 1
fi

echo "✓ Rust detected: $(rustc --version)"

# Check for Zebra installation
if ! command -v zebrad &> /dev/null; then
    echo "⚠️  Warning: Zebra (zebrad) not found in PATH"
    echo "Please install Zebra from: https://github.com/ZcashFoundation/zebra"
    echo ""
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo "✓ Zebra detected: $(zebrad --version)"
fi

# Build the proof generator
echo ""
echo "Building DePINZcash proof generator..."
cd prover
cargo build --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful!"
else
    echo "❌ Build failed. Please check the error messages above."
    exit 1
fi

cd ..

# Create necessary directories
mkdir -p proofs
mkdir -p config
echo "✓ Created directories"

# Create config if it doesn't exist
if [ ! -f "config/config.json" ]; then
    echo ""
    echo "📝 Creating configuration..."
    echo ""

    read -p "Enter your Solana wallet address: " solana_wallet
    read -p "Enter your Zcash address (t-addr or z-addr): " zcash_address
    read -p "Optional node identifier (press Enter to skip): " node_id

    cat > config/config.json <<EOF
{
  "solana_wallet": "$solana_wallet",
  "zcash_address": "$zcash_address",
  "node_id": ${node_id:+\"$node_id\"},
  "auto_submit": false,
  "api_endpoint": "https://depinzcash.io/api/submit",
  "api_key": null
}
EOF

    echo "✓ Configuration saved to config/config.json"
else
    echo "✓ Configuration already exists"
fi

# Make scripts executable
chmod +x scripts/*.sh

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "  1. Make sure your Zebra node is synced"
echo "  2. Run: ./generate_proof.sh"
echo "  3. Upload the generated proof to https://depinzcash.io/submit"
echo ""
