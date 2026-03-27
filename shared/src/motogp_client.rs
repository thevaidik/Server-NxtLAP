use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use crate::models::{RacingEvent, Series};

const CATEGORY_UUID: &str = "e8c110ad-64aa-4e8e-8a86-f2f152f6a942";

pub struct MotoGPClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct Season {
    id: String,
    year: i32,
}

#[derive(Debug, Deserialize)]
struct Event {
    id: String,
    name: String,
    short_name: String,
    circuit: Circuit,
    country: Country,
}

#[derive(Debug, Deserialize)]
struct Circuit {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Country {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Session {
    date: String,
    number: Option<i32>,
    #[serde(rename = "type")]
    session_type: String,
}

impl MotoGPClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.motogp.pulselive.com/motogp/v1/results".to_string(),
        }
    }

    pub async fn get_events(&self, year_str: &str) -> Result<Vec<RacingEvent>> {
        let year: i32 = year_str.parse().unwrap_or(2026);
        
        let seasons_url = format!("{}/seasons", self.base_url);
        let seasons: Vec<Season> = self.client.get(&seasons_url).send().await?.json().await?;
        
        let season_id = seasons.into_iter()
            .find(|s| s.year == year)
            .map(|s| s.id)
            .context("MotoGP season UUID not found for the given year")?;

        let events_url = format!("{}/events?seasonUuid={}", self.base_url, season_id);
        let events: Vec<Event> = self.client.get(&events_url).send().await?.json().await?;

        let mut racing_events = Vec::new();
        
        for event in events {
            let url = format!("{}/sessions?eventUuid={}&categoryUuid={}", self.base_url, event.id, CATEGORY_UUID);
            if let Ok(resp) = self.client.get(&url).send().await {
                if resp.status().is_success() {
                    let sessions: Vec<Session> = resp.json().await.unwrap_or_default();
                    
                    for session in sessions {
                        let session_name = match (session.session_type.as_str(), session.number) {
                            ("FP", Some(n)) => format!("Free Practice {}", n),
                            ("PR", _) => "Practice".to_string(),
                            ("Q", Some(n)) => format!("Qualifying {}", n),
                            ("SPR", _) => "Sprint".to_string(),
                            ("WUP", _) => "Warm Up".to_string(),
                            ("RAC", _) => "Race".to_string(),
                            (t, Some(n)) => format!("{} {}", t, n),
                            (t, None) => t.to_string(),
                        };
                        
                        let event_name = format!("{} {}", event.short_name, session_name);
                        
                        let parsed_date = DateTime::parse_from_rfc3339(&session.date)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc));
                            
                        if let Some(date) = parsed_date {
                            let ttl = (Utc::now() + chrono::Duration::days(30)).timestamp();
                            racing_events.push(RacingEvent {
                                id: format!("{}-{}", event.id, session.session_type),
                                series: Series::MotoGP,
                                event_name,
                                circuit: event.circuit.name.clone(),
                                date,
                                country: event.country.name.clone(),
                                season: year_str.to_string(),
                                round: None,
                                description: None,
                                ttl,
                            });
                        }
                    }
                }
            }
        }

        tracing::info!("Successfully fetched {} MotoGP sessions from PulseLive API", racing_events.len());
        Ok(racing_events)
    }
}
