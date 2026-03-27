use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::collections::HashMap;
use crate::models::{OpenF1Meeting, OpenF1Session, OpenF1DriverStandingRaw, OpenF1TeamStandingRaw, F1DriverStanding, F1TeamStanding, F1Standings, RacingEvent, Series};

pub struct OpenF1Client {
    client: Client,
    base_url: String,
}

impl OpenF1Client {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.openf1.org/v1".to_string(),
        }
    }

    pub async fn get_events(&self, year: &str) -> Result<Vec<RacingEvent>> {
        let meetings_url = format!("{}/meetings?year={}", self.base_url, year);
        let sessions_url = format!("{}/sessions?year={}", self.base_url, year);

        tracing::info!("Fetching OpenF1 meetings from {}", meetings_url);
        let meetings_resp = self.client.get(&meetings_url).send().await.context("Failed to fetch meetings")?;
        let meetings: Vec<OpenF1Meeting> = meetings_resp.json().await.context("Failed to parse meetings")?;

        let mut meeting_map = HashMap::new();
        for m in meetings {
            meeting_map.insert(m.meeting_key, m.meeting_name);
        }

        tracing::info!("Fetching OpenF1 sessions from {}", sessions_url);
        let sessions_resp = self.client.get(&sessions_url).send().await.context("Failed to fetch sessions")?;
        let sessions: Vec<OpenF1Session> = sessions_resp.json().await.context("Failed to parse sessions")?;

        let mut racing_events = Vec::new();

        for s in sessions {
            let meeting_name = meeting_map
                .get(&s.meeting_key)
                .cloned()
                .unwrap_or_else(|| "Unknown Meeting".to_string());
            
            let event_name = format!("{} {}", meeting_name, s.session_name)
                .trim()
                .to_string();

            let date_str = s.date_start.clone();
            
            let parsed_date = DateTime::parse_from_rfc3339(&date_str)
                .ok()
                .map(|dt| dt.with_timezone(&Utc));

            if let Some(date) = parsed_date {
                let ttl = (Utc::now() + chrono::Duration::days(30)).timestamp();
                
                racing_events.push(RacingEvent {
                    id: s.session_key.to_string(),
                    series: Series::Formula1,
                    event_name,
                    circuit: s.circuit_short_name,
                    date,
                    country: s.country_name,
                    season: s.year.to_string(),
                    round: None,
                    description: None,
                    ttl,
                });
            } else {
                tracing::warn!("Failed to parse date {} for event {}", date_str, event_name);
            }
        }

        tracing::info!("Successfully parsed {} OpenF1 events", racing_events.len());
        Ok(racing_events)
    }

    pub async fn get_standings(&self) -> Result<F1Standings> {
        let drivers_url = format!("{}/championship_drivers?session_key=latest", self.base_url);
        let teams_url = format!("{}/championship_teams?session_key=latest", self.base_url);

        tracing::info!("Fetching F1 driver standings");
        let drivers_raw: Vec<OpenF1DriverStandingRaw> = self.client
            .get(&drivers_url)
            .send()
            .await
            .context("Failed to fetch driver standings")?
            .json()
            .await
            .context("Failed to parse driver standings")?;

        tracing::info!("Fetching F1 team standings");
        let teams_raw: Vec<OpenF1TeamStandingRaw> = self.client
            .get(&teams_url)
            .send()
            .await
            .context("Failed to fetch team standings")?
            .json()
            .await
            .context("Failed to parse team standings")?;

        let session_key = drivers_raw.first().map(|d| d.session_key).unwrap_or(0);

        let mut drivers: Vec<F1DriverStanding> = drivers_raw
            .into_iter()
            .map(|d| F1DriverStanding {
                driver_number: d.driver_number,
                position: d.position_current,
                points: d.points_current,
            })
            .collect();
        drivers.sort_by_key(|d| d.position);

        let mut constructors: Vec<F1TeamStanding> = teams_raw
            .into_iter()
            .map(|t| F1TeamStanding {
                team_name: t.team_name,
                position: t.position_current,
                points: t.points_current,
            })
            .collect();
        constructors.sort_by_key(|t| t.position);

        let ttl = (chrono::Utc::now() + chrono::Duration::days(7)).timestamp();

        tracing::info!("Successfully fetched F1 standings: {} drivers, {} constructors", drivers.len(), constructors.len());

        Ok(F1Standings {
            drivers,
            constructors,
            session_key,
            updated_at: chrono::Utc::now(),
            ttl,
        })
    }
}
