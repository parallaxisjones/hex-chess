#!/bin/bash

# Build script for Hex Chess project

set -e

echo "🔨 Building Hex Chess project..."

# Build core library
echo "📦 Building core library..."
cargo build -p hex-chess-core

# Build signaling server
echo "🌐 Building signaling server..."
cargo build -p hex-chess-signaling

# Test core library
echo "🧪 Testing core library..."
cargo test -p hex-chess-core

echo "✅ Build completed successfully!"
echo ""
echo "To run the signaling server:"
echo "  cargo run -p hex-chess-signaling"
echo ""
echo "To build the WASM game (requires trunk):"
echo "  cd crates/game && trunk build"
echo ""
echo "To enter development shell:"
echo "  nix develop"
