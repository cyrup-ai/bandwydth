#!/bin/bash

# Production bandwidth monitoring script
# This script runs the fully functional bandwidth monitoring example

set -e

echo "🚀 Starting production bandwidth monitoring example..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: This script must be run from the project root directory"
    exit 1
fi

# Build the project first
echo "🔨 Building project..."
cargo build --release --example fully-functional-bandwidth

# Check if build succeeded
if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"

# Check if we have the required binary
BINARY_PATH="target/release/examples/fully-functional-bandwidth"
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Error: Binary not found at $BINARY_PATH"
    exit 1
fi

echo "🎯 Found binary at: $BINARY_PATH"

# Run the example
echo "🏃 Running production bandwidth monitoring example..."
echo "📋 This will:"
echo "   - Check for elevated privileges"
echo "   - Launch Rio terminal if needed for sudo access"
echo "   - Initialize packet sniffers on all network interfaces"
echo "   - Display real-time bandwidth statistics"
echo "   - Show packet capture details"
echo ""
echo "⚠️  Note: This requires elevated privileges for packet capture"
echo "🔐 You may be prompted for your password"
echo ""

# Run the example
./"$BINARY_PATH"

echo "✅ Production example completed!"