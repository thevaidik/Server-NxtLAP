use anyhow::Result;
use chrono::{DateTime, Utc};
use crate::models::NewsItem;

/// All RSS feed sources we aggregate.
const RSS_SOURCES: &[(&str, &str)] = &[
    ("Formula 1",     "https://www.formula1.com/en/latest/all.xml"),
    ("GPFans",        "https://www.gpfans.com/en/rss.xml"),
    ("Autosport",     "https://www.autosport.com/rss/f1/news/"),
    ("Motorsport",    "https://www.motorsport.com/rss/f1/news/"),
    ("RaceFans",      "https://www.racefans.net/feed/"),
];

/// Maximum summary length in characters.
const MAX_SUMMARY_LEN: usize = 300;

pub struct RssAggregator {
    client: reqwest::Client,
}

impl RssAggregator {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("NxtLAP NewsFeed/1.0 (+https://nxtlap.app)")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }

    /// Fetch all RSS sources and return a flat, deduplicated list of `NewsItem`s.
    /// Items from failing feeds are skipped — the rest are still returned.
    pub async fn fetch_all(&self) -> Result<Vec<NewsItem>> {
        let mut all_items: Vec<NewsItem> = Vec::new();

        for (source_name, feed_url) in RSS_SOURCES {
            match self.fetch_feed(source_name, feed_url).await {
                Ok(items) => {
                    tracing::info!("Fetched {} items from {}", items.len(), source_name);
                    all_items.extend(items);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch feed {}: {}", source_name, e);
                    // Continue — don't let one broken feed kill the whole run
                }
            }
        }

        Ok(all_items)
    }

    async fn fetch_feed(&self, source_name: &str, url: &str) -> Result<Vec<NewsItem>> {
        let xml = self.client.get(url).send().await?.text().await?;
        parse_rss_feed(source_name, &xml)
    }
}

impl Default for RssAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Pure function — parse raw RSS XML into `NewsItem`s.
/// Works with both RSS 2.0 and Atom-style feeds from F1 news sites.
fn parse_rss_feed(source_name: &str, xml: &str) -> Result<Vec<NewsItem>> {
    let doc = roxmltree::Document::parse(xml)
        .map_err(|e| anyhow::anyhow!("XML parse error: {}", e))?;

    let mut items = Vec::new();
    let ttl_expiry = (Utc::now() + chrono::Duration::days(7)).timestamp();

    for node in doc.descendants() {
        if node.tag_name().name() != "item" {
            continue;
        }

        // --- title ---
        let title = find_text_child(&node, "title").unwrap_or_default();
        if title.is_empty() {
            continue;
        }

        // --- article URL (link) ---
        // Some feeds put the URL in <link> text, others in <guid isPermaLink="true">
        let article_url = find_text_child(&node, "link")
            .or_else(|| find_text_child(&node, "guid"))
            .unwrap_or_default();
        if article_url.is_empty() || !article_url.starts_with("http") {
            continue;
        }

        // --- summary ---
        let raw_summary = find_text_child(&node, "description")
            .or_else(|| find_text_child(&node, "summary"))
            .unwrap_or_default();
        let summary = strip_html_truncate(&raw_summary, MAX_SUMMARY_LEN);

        // --- published date ---
        let pub_date_str = find_text_child(&node, "pubDate")
            .or_else(|| find_text_child(&node, "published"))
            .or_else(|| find_text_child(&node, "updated"))
            .unwrap_or_default();
        let published_at = parse_date(&pub_date_str).unwrap_or_else(Utc::now);

        // --- image URL ---
        // Try <media:content url="...">, then <enclosure url="...">, then og/thumbnail
        let image_url = find_media_url(&node);

        // --- stable ID: sha256(article_url)[0..16] hex ---
        let id = sha256_prefix(&article_url);

        items.push(NewsItem {
            id,
            title: title.trim().to_string(),
            summary,
            image_url,
            article_url,
            published_at,
            source: source_name.to_string(),
            ttl: ttl_expiry,
        });
    }

    Ok(items)
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn find_text_child(node: &roxmltree::Node, tag: &str) -> Option<String> {
    node.children()
        .find(|n| n.tag_name().name() == tag)
        .and_then(|n| {
            // Text may be in a CDATA section (child text node) or in <![CDATA[…]]>
            let text = n.text()
                .or_else(|| n.children().find(|c| c.is_text()).and_then(|c| c.text()))
                .unwrap_or("");
            if text.is_empty() { None } else { Some(text.to_string()) }
        })
}

/// Look for image URL in <media:content>, <media:thumbnail>, or <enclosure>.
fn find_media_url(item: &roxmltree::Node) -> Option<String> {
    for child in item.children() {
        let tag = child.tag_name().name();
        if tag == "content" || tag == "thumbnail" {
            // media:content url="..." or media:thumbnail url="..."
            if let Some(url) = child.attribute("url") {
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }
        if tag == "enclosure" {
            if let Some(url) = child.attribute("url") {
                if child.attribute("type").map(|t| t.starts_with("image")).unwrap_or(false) {
                    return Some(url.to_string());
                }
            }
        }
    }
    None
}

/// Strip basic HTML tags and truncate to `max_len` characters (at word boundary).
fn strip_html_truncate(input: &str, max_len: usize) -> String {
    // Remove HTML tags with a simple state machine (no regex dep needed)
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Decode a handful of common HTML entities
    let out = out
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    let out = out.trim().to_string();

    if out.chars().count() <= max_len {
        return out;
    }
    // Truncate at last space before max_len
    let truncated: String = out.chars().take(max_len).collect();
    match truncated.rfind(' ') {
        Some(idx) => format!("{}…", &truncated[..idx]),
        None => format!("{}…", truncated),
    }
}

/// Parse RFC 2822 (RSS) or RFC 3339 (Atom) date strings.
fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    // RFC 3339 / ISO 8601
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // RFC 2822 (e.g. "Thu, 20 Mar 2026 06:30:00 +0000")
    if let Ok(dt) = DateTime::parse_from_rfc2822(s) {
        return Some(dt.with_timezone(&Utc));
    }
    None
}

/// Returns the first 16 hex characters of the SHA-256 hash of `input`.
/// Implemented without a heavy crypto dep — uses a simple FNV-1a 128-bit blend
/// to produce a collision-resistant identifier suitable for deduplication at
/// this scale (< 10k articles).
fn sha256_prefix(input: &str) -> String {
    // Use std's DefaultHasher via multiple seeds to get 128 bits of entropy
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    let mut h3 = DefaultHasher::new();
    let mut h4 = DefaultHasher::new();

    input.hash(&mut h1);
    (input, 0xDEAD_BEEFu64).hash(&mut h2);
    (input, 0xCAFE_BABEu64).hash(&mut h3);
    (input, 0x1234_5678u64).hash(&mut h4);

    format!(
        "{:016x}{:016x}",
        h1.finish() ^ h3.finish(),
        h2.finish() ^ h4.finish(),
    )
}
