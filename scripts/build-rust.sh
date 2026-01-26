#!/bin/bash
set -e

echo "🦀 Building Rust Lambda functions..."
echo ""

# Check if cargo-lambda is installed
if ! command -v cargo-lambda &> /dev/null; then
    echo "❌ cargo-lambda not found. Please install it."
    exit 1
fi

# Build for ARM64 (cheaper and faster on Lambda)
echo "📦 Building data-fetcher..."
cargo lambda build --release --arm64 -p data-fetcher

echo "📦 Building api-handler..."
cargo lambda build --release --arm64 -p api-handler

echo ""
echo "✅ Build complete!"
echo ""
echo "Binary sizes:"
ls -lh target/lambda/data-fetcher/bootstrap | awk '{print $9, $5}'
ls -lh target/lambda/api-handler/bootstrap | awk '{print $9, $5}'
echo ""
echo "Ready to deploy!"
