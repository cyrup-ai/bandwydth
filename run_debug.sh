#!/bin/bash
echo "🔍 Running network interface diagnostic..."
echo "Building example..."
cargo build --example debug_interfaces
echo ""
echo "Running with elevated privileges..."
sudo RUST_LOG=info ./target/debug/examples/debug_interfaces