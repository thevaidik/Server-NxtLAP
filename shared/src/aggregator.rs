use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use crate::models::{ApiEvent, RacingEvent, Series};
use crate::thesportsdb_client::TheSportsDBClient;

pub struct RacingAggregator {
    client: TheSportsDBClient,
}

impl RacingAggregator {
    pub fn new(api_key: String) -> Self {
        Self {
            client: TheSportsDBClient::new(api_key),
        }
    }

    pub async fn fetch_all_series(&self) -> Result<Vec<RacingEvent>> {
        let mut all_events = Vec::new();

        for series in Series::all() {
            match self.fetch_series_events(series).await {
                Ok(events) => {
                    tracing::info!("Fetched {} events for {:?}", events.len(), series);
                    all_events.extend(events);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch {:?}: {}", series, e);
                    // Continue with other series even if one fails
                }
            }
        }

        tracing::info!("Total events fetched: {}", all_events.len());
        Ok(all_events)
    }

    async fn fetch_series_events(&self, series: Series) -> Result<Vec<RacingEvent>> {
        // Fetch full 2025 season (get_next_events now uses eventsseason.php)
        let all_events = self.client.get_next_events(series, 0).await?;

        // Convert API events to RacingEvent

        // Convert API events to RacingEvent
        let racing_events: Vec<RacingEvent> = all_events
            .into_iter()
            .filter_map(|api_event| self.convert_api_event(api_event, series))
            .collect();

        Ok(racing_events)
    }

    fn convert_api_event(&self, api_event: ApiEvent, series: Series) -> Option<RacingEvent> {
        // Parse date
        let date = api_event
            .date
            .as_ref()
            .and_then(|d| DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", d)).ok())
            .map(|dt| dt.with_timezone(&Utc))?;

        // Calculate TTL (7 days from now)
        let ttl = (Utc::now() + chrono::Duration::days(7)).timestamp();

        Some(RacingEvent {
            id: api_event.id,
            series,
            event_name: api_event.event_name,
            circuit: api_event.circuit.unwrap_or_else(|| "Unknown".to_string()),
            date,
            country: api_event.country.unwrap_or_else(|| "Unknown".to_string()),
            season: api_event.season.unwrap_or_else(|| "2024".to_string()),
            round: api_event.round.and_then(|r| r.parse().ok()),
            description: api_event.description,
            ttl,
        })
    }
}
