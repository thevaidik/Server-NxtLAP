# Racing Data Server Architecture

## Overview
Serverless racing data aggregation API using AWS Lambda, DynamoDB, and EventBridge.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     AWS Cloud                                │
│                                                              │
│  ┌──────────────┐                                           │
│  │ EventBridge  │ (Triggers every 3 days)                   │
│  │   Schedule   │                                           │
│  └──────┬───────┘                                           │
│         │                                                    │
│         │ Invoke                                            │
│         ▼                                                    │
│  ┌──────────────────────┐                                   │
│  │  data-fetcher        │                                   │
│  │  Lambda Function     │                                   │
│  │  (Rust - 8MB)        │                                   │
│  │                      │                                   │
│  │  1. Fetch from       │                                   │
│  │     TheSportsDB API  │◀────────┐                        │
│  │  2. Aggregate events │         │                        │
│  │  3. Store in DynamoDB│         │ TheSportsDB API        │
│  └──────────┬───────────┘         │ (External)             │
│             │                      │                        │
│             │ Write (~200 events)  │                        │
│             ▼                      │                        │
│  ┌──────────────────────┐         │                        │
│  │    DynamoDB          │         │                        │
│  │  RacingEvents Table  │         │                        │
│  │                      │         │                        │
│  │  - id (Primary Key)  │         │                        │
│  │  - data (JSON)       │         │                        │
│  │  - series            │         │                        │
│  │  - date              │         │                        │
│  │  - ttl (7 days)      │         │                        │
│  └──────────┬───────────┘         │                        │
│             │                      │                        │
│             │ Read                 │                        │
│             ▼                      │                        │
│  ┌──────────────────────┐         │                        │
│  │   api-handler        │         │                        │
│  │   Lambda Function    │         │                        │
│  │   (Rust - 8MB)       │         │                        │
│  │                      │         │                        │
│  │  GET /health         │         │                        │
│  │  GET /races          │         │                        │
│  │  GET /races/upcoming │         │                        │
│  │  GET /races/{series} │         │                        │
│  └──────────┬───────────┘         │                        │
│             │                      │                        │
│             │ Lambda Function URL  │                        │
│             ▼                      │                        │
│  ┌──────────────────────┐         │                        │
│  │   Public HTTPS       │         │                        │
│  │   Endpoint           │         │                        │
│  └──────────────────────┘         │                        │
│                                    │                        │
│                                    │                        │
│         Your iOS App ──────────────┘                        │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## Components

### 1. Data Fetcher Lambda
**Trigger:** EventBridge Schedule (every 3 days)
**Function:** Fetch and cache racing data

**Process:**
1. Triggered by EventBridge every 3 days
2. Fetches data from TheSportsDB API for 9 racing series:
   - Formula 1
   - IndyCar
   - MotoGP
   - IMSA
   - Super GT
   - British GT
   - BTCC
   - V8 Supercars
   - WRC
3. Gets ~25 events per series (15 upcoming + 10 past)
4. Aggregates ~200 total events
5. Stores each event in DynamoDB with:
   - Unique ID
   - Event details (name, circuit, date, country)
   - Series information
   - TTL (7 days from now)
6. Old events auto-delete after 7 days (DynamoDB TTL)

**Configuration:**
- Memory: 1024 MB
- Timeout: 300 seconds (5 minutes)
- Runtime: Rust (provided.al2)
- Architecture: ARM64

### 2. API Handler Lambda
**Trigger:** HTTP requests via Lambda Function URL
**Function:** Serve racing data to clients

**Endpoints:**
- `GET /health` → `{"status": "ok"}`
- `GET /races` → All cached races
- `GET /races/upcoming` → Only future races
- `GET /races/formula1` → F1 races only
- `GET /races/indycar` → IndyCar races only
- ... (all 9 series)

**Process:**
1. Receives HTTP request
2. Reads from DynamoDB (fast, cached data)
3. Filters based on endpoint
4. Returns JSON response with CORS headers

**Configuration:**
- Memory: 512 MB
- Timeout: 30 seconds
- Runtime: Rust (provided.al2)
- Architecture: ARM64
- Public access via Function URL

### 3. DynamoDB Table
**Name:** RacingEvents
**Primary Key:** id (String)

**Attributes:**
- `id` - Unique event ID
- `data` - Full event JSON
- `series` - Racing series name
- `date` - Event date (ISO 8601)
- `ttl` - Unix timestamp (auto-delete after 7 days)

**Billing:** Pay-per-request (free tier: 25 GB storage, 25 RCU/WCU)

## Data Flow

### Initial Setup (One-time)
```
1. Deploy Lambda functions
2. Create DynamoDB table
3. Set up EventBridge schedule
4. Get Function URL for API
```

### Every 3 Days (Automatic)
```
EventBridge → data-fetcher → TheSportsDB API → Aggregate → DynamoDB
```

### On API Request (Real-time)
```
iOS App → Function URL → api-handler → DynamoDB → JSON Response
```

## Cost Breakdown

### Free Tier (First 12 months)
- **Lambda:** 1M requests/month, 400,000 GB-seconds
- **DynamoDB:** 25 GB storage, 25 RCU/WCU
- **Data Transfer:** 1 GB/month

### Expected Usage
- **data-fetcher:** Runs 10 times/month (every 3 days)
- **api-handler:** ~1,000 requests/month (from iOS app)
- **DynamoDB:** ~200 items, ~1 MB storage

### Monthly Cost
- **Lambda:** $0 (well within free tier)
- **DynamoDB:** $0 (well within free tier)
- **Function URL:** $0 (no charge)
- **TheSportsDB API:** $0-9 (optional premium)

**Total: $0-9/month**

## Deployment Status

✅ **Local:** Clean, all Swift removed
✅ **AWS:** Clean, all resources deleted
🚀 **Ready:** To deploy Rust version

## Next Steps

1. Install cargo-lambda
2. Create DynamoDB table
3. Deploy Lambda functions
4. Create EventBridge schedule
5. Test API endpoints
