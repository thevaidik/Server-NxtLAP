#!/bin/bash
set -e

echo "🚀 Deploying Racing Data Server (Rust) to AWS..."
echo ""

REGION="${AWS_REGION:-us-east-1}"
API_KEY="${1:-1}"

echo "📍 Region: $REGION"
echo "🔑 API Key: $API_KEY"
echo ""

# Build first
echo "🔨 Building..."
./scripts/build-rust.sh

echo ""
echo "☁️  Deploying to AWS..."
echo ""

# Deploy data-fetcher
echo "1️⃣  Deploying data-fetcher..."
cargo lambda deploy data-fetcher \
  --region $REGION \
  --env-var THESPORTSDB_API_KEY=$API_KEY \
  --env-var DYNAMODB_TABLE_NAME=RacingEvents \
  --timeout 300 \
  --memory 1024

echo ""

# Deploy api-handler
echo "2️⃣  Deploying api-handler..."
cargo lambda deploy api-handler \
  --region $REGION \
  --env-var DYNAMODB_TABLE_NAME=RacingEvents \
  --timeout 30 \
  --memory 512

echo ""
echo "✅ Deployment complete!"
echo ""
echo "📊 Get function URLs:"
echo "  aws lambda get-function-url-config --function-name api-handler --region $REGION"
echo ""
echo "🧪 Test data fetcher:"
echo "  aws lambda invoke --function-name data-fetcher response.json --region $REGION"
echo ""
