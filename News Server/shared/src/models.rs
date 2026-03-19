use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single F1 news article item stored in DynamoDB and served via the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    /// Stable ID: first 16 hex chars of sha256(article_url)
    pub id: String,
    pub title: String,
    /// Truncated article description / excerpt (≤ 300 chars)
    pub summary: String,
    /// Hero image URL from RSS <media:content> or <enclosure> (nullable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Link to the original article
    pub article_url: String,
    /// ISO 8601 publication timestamp
    pub published_at: DateTime<Utc>,
    /// Human-readable source name, e.g. "Formula 1", "GPFans"
    pub source: String,
    /// DynamoDB TTL — Unix timestamp, 7 days from fetch time
    /// Excluded from the JSON API response
    #[serde(skip_serializing, skip_deserializing)]
    pub ttl: i64,
}
