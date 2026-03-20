use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client as DynamoDBClient;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::Value;
use shared::{DynamoDBService, OpenF1Client, RacingAggregator};
use tracing::info;

async fn function_handler(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    info!("Starting data fetcher");

    // Get environment variables
    let api_key = std::env::var("THESPORTSDB_API_KEY")
        .unwrap_or_else(|_| "1".to_string());
    let table_name = std::env::var("DYNAMODB_TABLE_NAME")
        .unwrap_or_else(|_| "RacingEvents".to_string());

    info!("Using table: {}", table_name);

    // Initialize AWS SDK
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamodb_client = DynamoDBClient::new(&config);
    let db_service = DynamoDBService::new(dynamodb_client, table_name);

    // Fetch racing data
    let aggregator = RacingAggregator::new(api_key);
    let events = aggregator.fetch_all_series().await?;

    info!("Fetched {} total events", events.len());

    // Store in DynamoDB
    db_service.put_events(events.clone()).await?;

    info!("Successfully stored {} events in DynamoDB", events.len());

    // Fetch and store F1 standings
    let openf1_client = OpenF1Client::new();
    match openf1_client.get_standings().await {
        Ok(standings) => {
            db_service.put_standings(&standings).await?;
            info!("Successfully stored F1 standings (session_key: {})", standings.session_key);
        }
        Err(e) => {
            // Don't fail the whole function if standings fetch fails
            tracing::warn!("Failed to fetch F1 standings: {}", e);
        }
    }

    Ok(serde_json::json!({
        "statusCode": 200,
        "message": format!("Fetched and stored {} events", events.len())
    }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
