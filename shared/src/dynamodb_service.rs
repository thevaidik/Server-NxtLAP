use anyhow::{Context, Result};
use aws_sdk_dynamodb::Client;
use chrono::Utc;
use serde_json;
use crate::models::{RacingEvent, Series};

pub struct DynamoDBService {
    client: Client,
    table_name: String,
}

impl DynamoDBService {
    pub fn new(client: Client, table_name: String) -> Self {
        Self { client, table_name }
    }

    pub async fn put_event(&self, event: &RacingEvent) -> Result<()> {
        // Convert to JSON string for DynamoDB
        let json_str = serde_json::to_string(event)?;
        
        self.client
            .put_item()
            .table_name(&self.table_name)
            .item("id", aws_sdk_dynamodb::types::AttributeValue::S(event.id.clone()))
            .item("data", aws_sdk_dynamodb::types::AttributeValue::S(json_str))
            .item("series", aws_sdk_dynamodb::types::AttributeValue::S(format!("{:?}", event.series)))
            .item("date", aws_sdk_dynamodb::types::AttributeValue::S(event.date.to_rfc3339()))
            .item("ttl", aws_sdk_dynamodb::types::AttributeValue::N(event.ttl.to_string()))
            .send()
            .await
            .context("Failed to put item in DynamoDB")?;

        Ok(())
    }

    pub async fn put_events(&self, events: Vec<RacingEvent>) -> Result<()> {
        for event in events {
            self.put_event(&event).await?;
        }
        Ok(())
    }

    pub async fn get_all_events(&self) -> Result<Vec<RacingEvent>> {
        let result = self
            .client
            .scan()
            .table_name(&self.table_name)
            .send()
            .await
            .context("Failed to scan DynamoDB table")?;

        let items = result.items.unwrap_or_default();
        let mut events = Vec::new();

        for item in items {
            if let Some(aws_sdk_dynamodb::types::AttributeValue::S(json_str)) = item.get("data") {
                if let Ok(event) = serde_json::from_str::<RacingEvent>(json_str) {
                    events.push(event);
                }
            }
        }

        Ok(events)
    }

    pub async fn get_upcoming_events(&self) -> Result<Vec<RacingEvent>> {
        let now = Utc::now();
        let all_events = self.get_all_events().await?;

        let upcoming: Vec<RacingEvent> = all_events
            .into_iter()
            .filter(|event| event.date > now)
            .collect();

        Ok(upcoming)
    }

    pub async fn get_events_by_series(&self, series: Series) -> Result<Vec<RacingEvent>> {
        let all_events = self.get_all_events().await?;

        let filtered: Vec<RacingEvent> = all_events
            .into_iter()
            .filter(|event| event.series == series)
            .collect();

        Ok(filtered)
    }

    pub async fn delete_event(&self, id: &str) -> Result<()> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", aws_sdk_dynamodb::types::AttributeValue::S(id.to_string()))
            .send()
            .await
            .context("Failed to delete item from DynamoDB")?;

        Ok(())
    }
}
