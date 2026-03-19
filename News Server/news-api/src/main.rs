use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client as DynamoDBClient;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use shared::NewsDynamoDBService;
use tracing::info;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let path = event.uri().path();
    info!("Handling request: {}", path);

    let table_name = std::env::var("NEWS_TABLE_NAME")
        .unwrap_or_else(|_| "NxtLAPNews".to_string());

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamodb_client = DynamoDBClient::new(&config);
    let db_service = NewsDynamoDBService::new(dynamodb_client, table_name);

    let response = match path {
        "/health" => {
            info!("Health check");
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::json!({"status": "ok", "service": "NxtLAP News"}).to_string().into())?
        }

        "/news" => {
            // Optional ?limit=N query param, capped at 100, default 50
            let limit = parse_limit(event.uri().query(), 50, 100);
            info!("Getting latest {} news items", limit);

            let items = db_service.get_latest_news(limit).await?;
            info!("Retrieved {} items from database", items.len());

            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                // iOS app may cache this for 30 min — news is refreshed every 6h server-side
                .header("Cache-Control", "public, max-age=1800")
                .body(serde_json::to_string(&items)?.into())?
        }

        _ => {
            Response::builder()
                .status(404)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::json!({"error": "Not found"}).to_string().into())?
        }
    };

    Ok(response)
}

/// Parse `?limit=N` from the query string.
fn parse_limit(query: Option<&str>, default: usize, max: usize) -> usize {
    let Some(q) = query else { return default };
    for pair in q.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next() == Some("limit") {
            if let Some(val) = parts.next() {
                if let Ok(n) = val.parse::<usize>() {
                    return n.min(max);
                }
            }
        }
    }
    default
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
