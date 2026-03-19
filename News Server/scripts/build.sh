#!/bin/bash
set -e

echo "🦀 Building NxtLAP News Server (Rust)..."
echo ""

# Check if cargo-lambda is installed
if ! command -v cargo-lambda &> /dev/null; then
    echo "❌ cargo-lambda not found."
    echo "   Install: brew install cargo-lambda"
    exit 1
fi

# Build for ARM64 (Graviton2 — cheaper & faster on Lambda)
echo "📦 Building news-fetcher..."
cargo lambda build --release --arm64 -p news-fetcher

echo "📦 Building news-api..."
cargo lambda build --release --arm64 -p news-api

echo ""
echo "✅ Build complete!"
echo ""
echo "Binary sizes:"
ls -lh target/lambda/news-fetcher/bootstrap | awk '{print $9, $5}'
ls -lh target/lambda/news-api/bootstrap     | awk '{print $9, $5}'
echo ""
echo "Ready to deploy! Run ./scripts/deploy.sh"
