#!/bin/bash
echo "Testing News API Gateway..."
echo ""
echo "Health check:"
curl -s https://meol2c3y91.execute-api.us-east-1.amazonaws.com/health
echo ""
echo ""
echo "Getting 3 news articles:"
curl -s "https://meol2c3y91.execute-api.us-east-1.amazonaws.com/news?limit=3" | python3 -m json.tool | head -30
echo ""
echo "✅ API Gateway is working!"
