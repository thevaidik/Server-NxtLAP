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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesMetadata {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub category: String,
    pub description: String,
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

    pub fn metadata(&self) -> SeriesMetadata {
        match self {
            Series::Formula1 => SeriesMetadata {
                id: "formula1".to_string(),
                name: "Formula 1".to_string(),
                short_name: "F1".to_string(),
                category: "Open Wheel".to_string(),
                description: "The highest class of international racing for open-wheel single-seater formula racing cars.".to_string(),
            },
            Series::IndyCar => SeriesMetadata {
                id: "indycar".to_string(),
                name: "IndyCar Series".to_string(),
                short_name: "IndyCar".to_string(),
                category: "Open Wheel".to_string(),
                description: "The premier level of open-wheel racing in North America.".to_string(),
            },
            Series::MotoGP => SeriesMetadata {
                id: "motogp".to_string(),
                name: "MotoGP".to_string(),
                short_name: "MotoGP".to_string(),
                category: "Two-Wheel".to_string(),
                description: "The premier class of motorcycle road racing events held on road circuits.".to_string(),
            },
            Series::IMSA => SeriesMetadata {
                id: "imsa".to_string(),
                name: "WeatherTech SportsCar Championship".to_string(),
                short_name: "IMSA".to_string(),
                category: "Sports Car".to_string(),
                description: "The premier sports car racing series in the United States and Canada.".to_string(),
            },
            Series::SuperGT => SeriesMetadata {
                id: "supergt".to_string(),
                name: "Super GT".to_string(),
                short_name: "Super GT".to_string(),
                category: "Sports Car".to_string(),
                description: "The top level of sports car racing in Japan.".to_string(),
            },
            Series::BritishGT => SeriesMetadata {
                id: "britishgt".to_string(),
                name: "British GT Championship".to_string(),
                short_name: "British GT".to_string(),
                category: "Sports Car".to_string(),
                description: "A sports car racing series based in the United Kingdom featuring GT3 and GT4 cars.".to_string(),
            },
            Series::BTCC => SeriesMetadata {
                id: "btcc".to_string(),
                name: "British Touring Car Championship".to_string(),
                short_name: "BTCC".to_string(),
                category: "Touring Car".to_string(),
                description: "A touring car racing series held each year in the United Kingdom.".to_string(),
            },
            Series::V8Supercars => SeriesMetadata {
                id: "v8supercars".to_string(),
                name: "Supercars Championship".to_string(),
                short_name: "Supercars".to_string(),
                category: "Touring Car".to_string(),
                description: "The premier motorsport category in Australasia.".to_string(),
            },
            Series::WRC => SeriesMetadata {
                id: "wrc".to_string(),
                name: "World Rally Championship".to_string(),
                short_name: "WRC".to_string(),
                category: "Rally".to_string(),
                description: "The highest level of global competition in the motorsport discipline of rallying.".to_string(),
            },
        }
    }

    pub fn all_metadata() -> Vec<SeriesMetadata> {
        Self::all().into_iter().map(|s| s.metadata()).collect()
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

// F1 Standings models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct F1DriverStanding {
    pub driver_number: i32,
    pub position: i32,
    pub points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct F1TeamStanding {
    pub team_name: String,
    pub position: i32,
    pub points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct F1Standings {
    pub drivers: Vec<F1DriverStanding>,
    pub constructors: Vec<F1TeamStanding>,
    pub session_key: i64,
    pub updated_at: DateTime<Utc>,
    pub ttl: i64,
}

// OpenF1 raw API response models for standings
#[derive(Debug, Deserialize)]
pub struct OpenF1DriverStandingRaw {
    pub driver_number: i32,
    pub session_key: i64,
    pub position_current: i32,
    pub points_current: f64,
}

#[derive(Debug, Deserialize)]
pub struct OpenF1TeamStandingRaw {
    pub team_name: String,
    pub session_key: i64,
    pub position_current: i32,
    pub points_current: f64,
}

// OpenF1 API response models
#[derive(Debug, Deserialize)]
pub struct OpenF1Meeting {
    pub meeting_key: i64,
    pub meeting_name: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenF1Session {
    pub session_key: i64,
    pub session_type: String,
    pub session_name: String,
    pub date_start: String,
    pub meeting_key: i64,
    pub circuit_short_name: String,
    pub country_name: String,
    pub year: i32,
}
