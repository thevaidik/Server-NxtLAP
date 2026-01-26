# Racing Data Server (Rust) 🦀

A serverless racing data aggregation API built with Rust and AWS Lambda.

## Features

- **9 Racing Series:** Formula 1, IndyCar, MotoGP, IMSA, Super GT, British GT, BTCC, V8 Supercars, WRC
- **REST API:** Get all races, upcoming races, or filter by series
- **DynamoDB Caching:** 7-day TTL for efficient data storage
- **Serverless:** AWS Lambda with ARM64 for cost efficiency
- **Small Binaries:** ~5-10MB (vs 278MB Swift!)

## Architecture

```
┌─────────────────┐
│  EventBridge    │ (Every 3 days)
└────────┬────────┘
         │
         ▼
┌─────────────────┐      ┌──────────────┐
│  data-fetcher   │─────▶│  DynamoDB    │
│    Lambda       │      │ RacingEvents │
└─────────────────┘      └──────┬───────┘
                                │
                                │
┌─────────────────┐             │
│   API Gateway   │◀────────────┘
│    HTTP API     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   api-handler   │
│     Lambda      │
└─────────────────┘
```

## Project Structure

```
Server NxtLAP/
├── data-fetcher/       # EventBridge Lambda
│   ├── Cargo.toml
│   └── src/main.rs
├── api-handler/        # HTTP API Lambda
│   ├── Cargo.toml
│   └── src/main.rs
├── shared/             # Shared library
│   ├── Cargo.toml
│   └── src/
│       ├── models.rs
│       ├── thesportsdb_client.rs
│       ├── dynamodb_service.rs
│       └── aggregator.rs
└── Cargo.toml          # Workspace config
```

## Prerequisites

- Rust 1.70+
- cargo-lambda: `pip3 install cargo-lambda`
- AWS CLI configured
- DynamoDB table: `RacingEvents` (with `id` as primary key)

## Build

```bash
./scripts/build-rust.sh
```

This compiles both Lambda functions for ARM64 architecture.

## Deploy

```bash
./scripts/deploy-rust.sh <THESPORTSDB_API_KEY>
```

Or manually:

```bash
# Build
cargo lambda build --release --arm64

# Deploy data-fetcher
cargo lambda deploy data-fetcher \
  --region us-east-1 \
  --env-var THESPORTSDB_API_KEY=your_key \
  --env-var DYNAMODB_TABLE_NAME=RacingEvents \
  --timeout 300 \
  --memory 1024

# Deploy api-handler
cargo lambda deploy api-handler \
  --region us-east-1 \
  --env-var DYNAMODB_TABLE_NAME=RacingEvents \
  --timeout 30 \
  --memory 512
```

## API Endpoints

## 📚 API Documentation

**👉 [View Full API Guide](API_GUIDE.md)** for detailed endpoint usage.

### Quick Start
**Base URL:**
`https://YOUR_API_GATEWAY_URL`

- `GET /health` - Server status
- `GET /races` - All current season races
- `GET /races/{series}` - Filter by series (e.g., `formula1`, `motogp`)

## Testing Locally

```bash
# Start local Lambda server
cargo lambda watch

# Invoke data-fetcher
cargo lambda invoke data-fetcher \
  --data-file events/scheduled-event.json

# Test API
curl http://localhost:9000/health
```

## Environment Variables

### data-fetcher
- `THESPORTSDB_API_KEY` - TheSportsDB API key (required)
- `DYNAMODB_TABLE_NAME` - DynamoDB table name (default: RacingEvents)

### api-handler
- `DYNAMODB_TABLE_NAME` - DynamoDB table name (default: RacingEvents)

## Cost Estimate

### AWS Free Tier
- **Lambda:** 1M requests/month = $0
- **DynamoDB:** 25 GB storage + 25 RCU/WCU = $0
- **Lambda Function URL:** No charge

### Paid (if exceeding free tier)
- **Lambda:** $0.20 per 1M requests
- **DynamoDB:** $1.25 per million writes
- **TheSportsDB API:** $0-9/month

**Estimated monthly cost:** $0-9/month

## Performance

- **Cold start:** ~100-200ms (ARM64)
- **Warm response:** ~10-50ms
- **Binary size:** ~8MB per function
- **Memory usage:** ~50-100MB

## Development

```bash
# Check code
cargo check --workspace

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Lint
cargo clippy --all
```

## Deployment via SAM (Alternative)

If you prefer SAM over cargo-lambda:

```bash
sam build
sam deploy --guided
```

## License

MIT

## Credits

- Racing data from [TheSportsDB](https://www.thesportsdb.com/)
- Built with [cargo-lambda](https://www.cargo-lambda.info/)
