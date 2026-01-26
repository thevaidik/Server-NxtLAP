use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RacingEvent {
    pub id: String,
    pub series: Series,
    pub event_name: String,
    pub circuit: String,
    pub date: DateTime<Utc>,
    pub country: String,
    pub season: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub ttl: i64, // Unix timestamp for DynamoDB TTL
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Series {
    Formula1,
    IndyCar,
    MotoGP,
    IMSA,
    SuperGT,
    BritishGT,
    BTCC,
    V8Supercars,
    WRC,
}

impl Series {
    pub fn all() -> Vec<Series> {
        vec![
            Series::Formula1,
            Series::IndyCar,
            Series::MotoGP,
            Series::IMSA,
            Series::SuperGT,
            Series::BritishGT,
            Series::BTCC,
            Series::V8Supercars,
            Series::WRC,
        ]
    }

    pub fn thesportsdb_id(&self) -> &str {
        match self {
            Series::Formula1 => "4370",
            Series::IndyCar => "4373",      // Updated from 4387
            Series::MotoGP => "4407",       // Updated from 4391
            Series::IMSA => "4488",         // Updated from 4424
            Series::SuperGT => "4412",      // Updated from 4425
            Series::BritishGT => "4410",    // Updated from 4426
            Series::BTCC => "4372",         // Updated from 4427
            Series::V8Supercars => "4489",  // Updated from 4428
            Series::WRC => "4409",          // Updated from 4415
        }
    }

    pub fn from_str(s: &str) -> Option<Series> {
        match s.to_lowercase().as_str() {
            "formula1" | "f1" => Some(Series::Formula1),
            "indycar" => Some(Series::IndyCar),
            "motogp" => Some(Series::MotoGP),
            "imsa" => Some(Series::IMSA),
            "supergt" => Some(Series::SuperGT),
            "britishgt" => Some(Series::BritishGT),
            "btcc" => Some(Series::BTCC),
            "v8supercars" => Some(Series::V8Supercars),
            "wrc" => Some(Series::WRC),
            _ => None,
        }
    }
}

// TheSportsDB API response models
#[derive(Debug, Deserialize)]
pub struct EventsResponse {
    pub events: Option<Vec<ApiEvent>>,
}

#[derive(Debug, Deserialize)]
pub struct ApiEvent {
    #[serde(rename = "idEvent")]
    pub id: String,
    #[serde(rename = "strEvent")]
    pub event_name: String,
    #[serde(rename = "strVenue")]
    pub circuit: Option<String>,
    #[serde(rename = "dateEvent")]
    pub date: Option<String>,
    #[serde(rename = "strCountry")]
    pub country: Option<String>,
    #[serde(rename = "strSeason")]
    pub season: Option<String>,
    #[serde(rename = "intRound")]
    pub round: Option<String>,
    #[serde(rename = "strDescriptionEN")]
    pub description: Option<String>,
    #[serde(rename = "strTime")]
    pub time: Option<String>,
    #[serde(rename = "strTimestamp")]
    pub timestamp: Option<String>,
    #[serde(rename = "strSport")]
    pub sport: Option<String>,
}
