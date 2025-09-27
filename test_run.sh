#!/bin/bash
# Test script to run bandwydth and capture any crash output

echo "Building bandwydth..."
cargo build 2>&1

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo "Running bandwydth (will timeout after 5 seconds)..."
echo "NOTE: This will fail with privilege error, but we're testing for crashes"
echo ""

# Run with timeout to avoid hanging
timeout 5 ./target/debug/bandwydth 2>&1 || true

echo ""
echo "Test completed. Check output above for any panics or crashes."