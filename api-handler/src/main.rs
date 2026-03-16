use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client as DynamoDBClient;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use shared::{DynamoDBService, Series};
use tracing::info;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let path = event.uri().path();
    info!("Handling request: {}", path);

    // Get table name from environment
    let table_name = std::env::var("DYNAMODB_TABLE_NAME")
        .unwrap_or_else(|_| "RacingEvents".to_string());

    // Initialize DynamoDB
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let dynamodb_client = DynamoDBClient::new(&config);
    let db_service = DynamoDBService::new(dynamodb_client, table_name);

    // Route handling
    let response = match path {
        "/health" => {
            info!("Health check");
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::json!({"status": "ok"}).to_string().into())?
        }
        "/races" => {
            info!("Getting all races");
            let events = db_service.get_all_events().await?;
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::to_string(&events)?.into())?
        }
        "/races/upcoming" => {
            info!("Getting upcoming races");
            let events = db_service.get_upcoming_events().await?;
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::to_string(&events)?.into())?
        }
        "/series" => {
            info!("Getting all series metadata");
            let series_list = Series::all_metadata();
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Access-Control-Allow-Origin", "*")
                .body(serde_json::to_string(&series_list)?.into())?
        }
        path if path.starts_with("/races/") => {
            let series_name = path.trim_start_matches("/races/");
            info!("Getting races for series: {}", series_name);
            
            match Series::from_str(series_name) {
                Some(series) => {
                    let events = db_service.get_events_by_series(series).await?;
                    Response::builder()
                        .status(200)
                        .header("Content-Type", "application/json")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(serde_json::to_string(&events)?.into())?
                }
                None => {
                    Response::builder()
                        .status(404)
                        .header("Content-Type", "application/json")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(serde_json::json!({"error": "Series not found"}).to_string().into())?
                }
            }
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
