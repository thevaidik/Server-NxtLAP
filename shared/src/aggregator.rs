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
        // FILTER: Drop rogue events (Golf, Soccer, etc.) that sneak in via Free Tier
        if let Some(sport) = &api_event.sport {
            if sport != "Motorsport" {
                tracing::warn!("Dropping rogue event: {} (Sport: {})", api_event.event_name, sport);
                return None;
            }
        }

        // Parse date
        let date_str = if let Some(ts) = &api_event.timestamp {
            // Try using strTimestamp first (often format: YYYY-MM-DDTHH:mm:ss)
            format!("{}Z", ts) // Append Z assuming UTC if not present
        } else if let (Some(d), Some(t)) = (&api_event.date, &api_event.time) {
            // Combine dateEvent and strTime
            format!("{}T{}Z", d, t)
        } else if let Some(d) = &api_event.date {
            // Fallback to date only (midnight)
            format!("{}T00:00:00Z", d)
        } else {
            return None;
        };

        let date = DateTime::parse_from_rfc3339(&date_str) // Try standard ISO first
            .or_else(|_| DateTime::parse_from_rfc3339(&format!("{}Z", date_str.trim_end_matches('Z')))) // Retry robustly
            .ok()
            .map(|dt| dt.with_timezone(&Utc));

        // If parsing failed (e.g. empty strings), skip this event
        let date = date?;

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
