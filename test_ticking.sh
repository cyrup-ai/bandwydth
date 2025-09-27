#!/bin/bash
# Test script to verify auto-ticking is working

echo "Building the project..."
cargo build --release

echo ""
echo "Running bandwydth with debug logging..."
echo "Watch for tick events in the log output."
echo "You should see components updating automatically every 250ms."
echo ""
echo "Press Ctrl+C to exit."
echo ""

# Run with debug logging to see tick events
RUST_LOG=debug cargo run --release 2>&1 | grep -E "(tick|Tick|force_tick)"