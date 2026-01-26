use anyhow::{Context, Result};
use reqwest::Client;
use crate::models::{ApiEvent, EventsResponse, Series};

pub struct TheSportsDBClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl TheSportsDBClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://www.thesportsdb.com/api/v1/json".to_string(),
        }
    }

    pub async fn get_next_events(&self, series: Series, _count: u32) -> Result<Vec<ApiEvent>> {
        let league_id = series.thesportsdb_id();
        let current_year = chrono::Utc::now().format("%Y").to_string();
        
        // Use eventsseason.php to fetch the full current season (e.g., 2026)
        let url = format!(
            "{}/{}/eventsseason.php?id={}&s={}",
            self.base_url, self.api_key, league_id, current_year
        );

        tracing::info!("Fetching {} season events for {:?} from {}", current_year, series, url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to TheSportsDB")?;

        if !response.status().is_success() {
            anyhow::bail!("TheSportsDB API returned status: {}", response.status());
        }

        let events_response: EventsResponse = response
            .json()
            .await
            .context("Failed to parse TheSportsDB response")?;

        // Return all events for the season (filtering happens in aggregator if needed)
        let events = events_response.events.unwrap_or_default();
        
        tracing::info!("Fetched {} events for {:?} (2025 Season)", events.len(), series);
        Ok(events)
    }

    // get_last_events removed as we now fetch full season via get_next_events (eventsseason.php)
}
