#!/bin/bash
set -e

echo "🚀 Deploying Staging Racing Data Server (Rust) to AWS..."
echo ""

REGION="${AWS_REGION:-us-east-1}"
API_KEY="${1:-1}"

echo "📍 Region: $REGION"
echo "🔑 API Key: $API_KEY"
echo ""

# Build first
echo "🔨 Building staging binaries..."
./scripts/build-rust.sh

echo ""
echo "☁️  Deploying to AWS Staging..."
echo ""

# Deploy data-fetcher-staging
echo "1️⃣  Deploying data-fetcher-staging..."
cargo lambda deploy data-fetcher-staging \
  --region $REGION \
  --env-var THESPORTSDB_API_KEY=$API_KEY \
  --env-var DYNAMODB_TABLE_NAME=RacingEvents \
  --timeout 300 \
  --memory 1024

echo ""

# Deploy api-handler-staging
echo "2️⃣  Deploying api-handler-staging..."
cargo lambda deploy api-handler-staging \
  --region $REGION \
  --env-var DYNAMODB_TABLE_NAME=RacingEvents \
  --timeout 30 \
  --memory 512

echo ""
echo "✅ Staging Deployment complete!"
echo ""
echo "📊 Get function URLs:"
echo "  aws lambda get-function-url-config --function-name api-handler-staging --region $REGION"
echo ""
echo "🧪 Test data fetcher:"
echo "  aws lambda invoke --function-name data-fetcher-staging response.json --region $REGION"
echo ""
