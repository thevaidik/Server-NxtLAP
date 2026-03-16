# Deployment Guide

## Quick Deploy (3 Steps)

### 1. Configure AWS CLI (One-time)
```bash
aws configure
```

Enter:
- **AWS Access Key ID:** (from AWS Console → IAM)
- **AWS Secret Access Key:** (from AWS Console → IAM)
- **Default region:** `us-east-1` (or your preferred region)
- **Default output format:** `json`

### 1a. Configure DynamoDB TTL (One-time)
```bash
# Enable TTL on the 'ttl' attribute
aws dynamodb update-time-to-live \
    --table-name RacingEvents \
    --time-to-live-specification "Enabled=true, AttributeName=ttl" \
    --region us-east-1
```

### 2. Deploy
```bash
cd "/Users/vaidikdubey/Developer/NxtLAP Folder/Server NxtLAP"
./scripts/deploy.sh
```

When prompted:
- **TheSportsDB API Key:** `1` (for testing) or your premium key
- **Stack name:** `racing-data-server` (or any name you prefer)

**Wait:** 5-10 minutes for deployment to complete.

### 3. Get Your API Endpoint

After deployment, you'll see:
```
Outputs:
  APIEndpoint: https://abc123xyz.lambda-url.us-east-1.on.aws
```

**Save this URL!** You'll use it in your iOS app.

---

## Test Your Deployment

```bash
# Replace with your actual endpoint
API="https://YOUR-ENDPOINT.lambda-url.us-east-1.on.aws"

# 1. Health check
curl $API/health

# 2. Trigger data fetch
aws lambda invoke \
  --function-name RacingDataFetcher \
  --payload '{}' \
  response.json

# 3. Wait 1-2 minutes, then test
curl $API/races/upcoming
curl $API/races/formula1
```

---

## iOS App Integration

```swift
let apiEndpoint = "https://YOUR-ENDPOINT.lambda-url.us-east-1.on.aws"

struct RacingAPIClient {
    func fetchUpcomingRaces() async throws -> RacingScheduleResponse {
        let url = URL(string: "\(apiEndpoint)/races/upcoming")!
        let (data, _) = try await URLSession.shared.data(from: url)
        return try JSONDecoder().decode(RacingScheduleResponse.self, from: data)
    }
}
```

---

## Available Endpoints

- `GET /health` - Health check
- `GET /races` - All races from all series
- `GET /races/upcoming` - Only upcoming races
- `GET /races/{series}` - Races for specific series

**Series values:**
`formula1`, `indycar`, `motogp`, `imsa`, `super_gt`, `british_gt`, `btcc`, `v8_supercars`, `wrc`

---

## Monitoring

```bash
# View logs
aws logs tail /aws/lambda/RacingDataFetcher --follow
aws logs tail /aws/lambda/RacingDataAPI --follow

# Check DynamoDB
aws dynamodb scan --table-name RacingEvents --select COUNT
```

---

## Cost

**AWS Infrastructure:** $0/month (100% free tier)
- Lambda: Free (1M requests/month)
- DynamoDB: Free (25 RCU/WCU)
- Lambda Function URL: Free

**TheSportsDB API:**
- Free: $0/month (key "1")
- Premium: $9/month (recommended for production)

**Total: $0-9/month**

---

## Troubleshooting

### "No data returned"
```bash
# Manually trigger data fetch
aws lambda invoke --function-name RacingDataFetcher --payload '{}' response.json
# Wait 1-2 minutes, then try again
```

### "Stack already exists"
```bash
# Update existing stack
sam deploy --no-confirm-changeset
```

### Delete Everything
```bash
sam delete --stack-name racing-data-server
```

---

## That's It!

Your racing data server is ready to use. 🏎️
