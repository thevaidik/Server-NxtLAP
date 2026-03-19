#!/bin/bash
set -e

echo "🚀 Deploying NxtLAP News Server to AWS..."
echo ""

REGION="${AWS_REGION:-us-east-1}"
echo "📍 Region: $REGION"
echo ""

# Build first
echo "🔨 Building..."
./scripts/build.sh

echo ""
echo "☁️  Deploying to AWS..."
echo ""

# Deploy news-fetcher
echo "1️⃣  Deploying news-fetcher..."
cargo lambda deploy \
  --binary-name news-fetcher \
  --region "$REGION" \
  --env-var NEWS_TABLE_NAME=NxtLAPNews \
  --timeout 120 \
  --memory 256

echo ""

# Deploy news-api
echo "2️⃣  Deploying news-api..."
cargo lambda deploy \
  --binary-name news-api \
  --region "$REGION" \
  --env-var NEWS_TABLE_NAME=NxtLAPNews \
  --timeout 30 \
  --memory 256

echo ""
echo "✅ Deployment complete!"
echo ""
echo "📊 Get news-api Function URL:"
echo "  aws lambda get-function-url-config --function-name news-api --region $REGION"
echo ""
echo "🧪 Trigger a manual news fetch:"
echo "  aws lambda invoke --function-name news-fetcher --payload '{}' /tmp/news-out.json && cat /tmp/news-out.json"
echo ""
echo "🧪 Test the API:"
echo "  curl \$(aws lambda get-function-url-config --function-name news-api --region $REGION --query FunctionUrl --output text)news | python3 -m json.tool"
echo ""
