#!/bin/bash
REGION="${AWS_REGION:-us-east-1}"
TABLE_NAME="RacingEvents"
TTL_ATTRIBUTE="ttl"

echo "Running command: aws dynamodb update-time-to-live --table-name $TABLE_NAME --time-to-live-specification \"Enabled=true, AttributeName=$TTL_ATTRIBUTE\" --region $REGION"
aws dynamodb update-time-to-live \
    --table-name "$TABLE_NAME" \
    --time-to-live-specification "Enabled=true, AttributeName=$TTL_ATTRIBUTE" \
    --region "$REGION"
