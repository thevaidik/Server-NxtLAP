use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client as DynamoDBClient;
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::Value;
use shared::{NewsDynamoDBService, RssAggregator};
use tracing::info;

async fn function_handler(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    info!("News fetcher started");

    let table_name = std::env::var("NEWS_TABLE_NAME")
        .unwrap_or_else(|_| "NxtLAPNews".to_string());

    info!("Using table: {}", table_name);

    // Initialize AWS
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamodb_client = DynamoDBClient::new(&config);
    let db_service = NewsDynamoDBService::new(dynamodb_client, table_name);

    // Fetch all RSS feeds
    let aggregator = RssAggregator::new();
    let items = aggregator.fetch_all().await?;

    info!("Fetched {} total items from RSS feeds", items.len());

    // Store each item — conditionally skips duplicates
    let mut saved = 0usize;
    let mut skipped = 0usize;
    for item in &items {
        match db_service.save_news_item(item).await {
            Ok(_) => saved += 1,
            Err(e) => {
                tracing::error!("Failed to save item {}: {}", item.id, e);
                skipped += 1;
            }
        }
    }

    info!("Saved: {}, Skipped/duplicates: {}", saved, skipped);

    Ok(serde_json::json!({
        "statusCode": 200,
        "fetched": items.len(),
        "saved": saved,
        "skipped": skipped
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
