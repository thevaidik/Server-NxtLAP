use anyhow::{Context, Result};
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use chrono::Utc;
use crate::models::NewsItem;

pub struct NewsDynamoDBService {
    client: Client,
    table_name: String,
}

impl NewsDynamoDBService {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    /// Save a news item. Uses a conditional write to skip duplicates —
    /// if an item with the same `id` already exists, this is a no-op.
    pub async fn save_news_item(&self, item: &NewsItem) -> Result<()> {
        let json_str = serde_json::to_string(item)?;

        let result = self.client
            .put_item()
            .table_name(&self.table_name)
            // Primary key
            .item("id", AttributeValue::S(item.id.clone()))
            // Full blob — same pattern as existing RacingEvents table
            .item("data", AttributeValue::S(json_str))
            // Indexed attributes for sorting/filtering
            .item("published_at", AttributeValue::S(item.published_at.to_rfc3339()))
            .item("source", AttributeValue::S(item.source.clone()))
            // DynamoDB TTL attribute
            .item("ttl", AttributeValue::N(item.ttl.to_string()))
            // Skip if already exists — idempotent
            .condition_expression("attribute_not_exists(id)")
            .send()
            .await;

        match result {
            Ok(_) => Ok(()),
            // ConditionalCheckFailedException means the item already exists — that's fine
            Err(e) => {
                let service_err = e.into_service_error();
                if service_err.is_conditional_check_failed_exception() {
                    tracing::debug!("Item already exists, skipping: {}", item.id);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("DynamoDB put_item failed: {:?}", service_err))
                }
            }
        }
    }

    /// Fetch the latest N news items, sorted by published_at descending.
    /// Scans the whole table (same approach as existing RacingEvents),
    /// which is fine under free-tier scale (≤ a few hundred rows at any time,
    /// older items auto-expire via TTL).
    pub async fn get_latest_news(&self, limit: usize) -> Result<Vec<NewsItem>> {
        let result = self.client
            .scan()
            .table_name(&self.table_name)
            .send()
            .await
            .context("Failed to scan NxtLAPNews table")?;

        let items = result.items.unwrap_or_default();
        tracing::info!("Scanned {} items from DynamoDB", items.len());
        let mut news: Vec<NewsItem> = Vec::new();

        for item in items {
            if let Some(AttributeValue::S(json_str)) = item.get("data") {
                match serde_json::from_str::<NewsItem>(json_str) {
                    Ok(mut news_item) => {
                        // ttl is excluded from JSON so restore it from the dedicated attribute
                        if let Some(AttributeValue::N(ttl_str)) = item.get("ttl") {
                            if let Ok(ttl) = ttl_str.parse::<i64>() {
                                news_item.ttl = ttl;
                            }
                        }
                        // Filter out items past TTL that DynamoDB hasn't cleaned up yet
                        let now = Utc::now().timestamp();
                        tracing::debug!("Item TTL: {}, Now: {}, Valid: {}", news_item.ttl, now, news_item.ttl > now);
                        if news_item.ttl > now {
                            news.push(news_item);
                        } else {
                            tracing::debug!("Filtered out expired item: {}", news_item.id);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse news item: {}", e);
                    }
                }
            }
        }

        tracing::info!("Parsed {} valid news items", news.len());

        // Sort newest first
        news.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        news.truncate(limit);

        Ok(news)
    }
}
