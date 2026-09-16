use crate::core::adblock::AdBlocker;
use crate::core::security::{fetch_bounded_content, validate_url_ssrf, SecurityError, MAX_HTTP_PAYLOAD_SIZE};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedItem {
    pub id: String,
    pub title: String,
    pub link: String,
    pub summary: String,
    pub published_date: Option<String>,
    pub source_name: String,
    pub source_id: String,
    pub category: String,
    pub is_read: bool,
    pub reading_time_mins: usize,
}

pub struct FeedParser;

impl FeedParser {
    /// Discovers the RSS/Atom feed URL for a given domain or website URL.
    pub async fn discover_feed_url(site_url: &str) -> Result<String, SecurityError> {
        let base_url = validate_url_ssrf(site_url)?;

        // If the URL already ends with feed/rss extensions, return directly
        let path = base_url.path().to_lowercase();
        if path.ends_with(".xml")
            || path.ends_with("/feed")
            || path.ends_with("/rss")
            || path.ends_with("/feed/")
            || path.ends_with("/rss/")
        {
            return Ok(base_url.to_string());
        }

        // Fetch homepage HTML (up to 1 MiB)
        let html_content = match fetch_bounded_content(base_url.as_str(), 1024 * 1024, 6).await {
            Ok(c) => c,
            Err(_) => {
                // If homepage fetch fails, try fallback endpoints directly
                return Self::probe_fallback_endpoints(&base_url).await;
            }
        };

        // If homepage itself returned XML feed
        let trimmed = html_content.trim();
        if trimmed.starts_with("<?xml") || trimmed.contains("<rss") || trimmed.contains("<feed") {
            return Ok(base_url.to_string());
        }

        // Parse HTML looking for <link rel="alternate" type="application/rss+xml" ...>
        let doc = Html::parse_document(&html_content);
        if let Ok(sel) = Selector::parse("link[rel='alternate']") {
            for link in doc.select(&sel) {
                let type_attr = link.value().attr("type").unwrap_or("").to_lowercase();
                if type_attr.contains("rss") || type_attr.contains("atom") || type_attr.contains("xml") {
                    if let Some(href) = link.value().attr("href") {
                        if let Ok(resolved) = base_url.join(href) {
                            return Ok(resolved.to_string());
                        }
                    }
                }
            }
        }

        // Fallback: Probe common feed paths
        Self::probe_fallback_endpoints(&base_url).await
    }

    async fn probe_fallback_endpoints(base_url: &Url) -> Result<String, SecurityError> {
        let candidates = [
            "feed",
            "rss",
            "rss.xml",
            "feed.xml",
            "atom.xml",
            "index.xml",
            "feed/",
            "rss/",
        ];

        for path in &candidates {
            if let Ok(target) = base_url.join(path) {
                if let Ok(content) = fetch_bounded_content(target.as_str(), 64 * 1024, 4).await {
                    let trimmed = content.trim();
                    if trimmed.starts_with("<?xml") || trimmed.contains("<rss") || trimmed.contains("<feed") {
                        return Ok(target.to_string());
                    }
                }
            }
        }

        Err(SecurityError::Network(format!(
            "{} adresi icin gecerli bir RSS/Atom akisi bulunamadi",
            base_url
        )))
    }

    /// Fetches and parses feed items from an RSS/Atom endpoint.
    pub async fn fetch_feed_items(
        feed_url: &str,
        source_id: &str,
        source_name: &str,
        category: &str,
    ) -> Result<Vec<FeedItem>, SecurityError> {
        let xml_text = fetch_bounded_content(feed_url, MAX_HTTP_PAYLOAD_SIZE, 8).await?;
        Self::parse_xml(&xml_text, source_id, source_name, category)
    }

    /// Parses RSS/Atom XML text into FeedItem structs.
    pub fn parse_xml(
        xml_text: &str,
        source_id: &str,
        source_name: &str,
        category: &str,
    ) -> Result<Vec<FeedItem>, SecurityError> {
        let mut reader = Reader::from_str(xml_text);
        reader.config_mut().trim_text(true);

        let mut items = Vec::new();
        let mut in_item = false;
        let mut in_entry = false;

        let mut current_tag = String::new();
        let mut cur_title = String::new();
        let mut cur_link = String::new();
        let mut cur_desc = String::new();
        let mut cur_date = String::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let name_lower = name.to_lowercase();

                    if name_lower == "item" {
                        in_item = true;
                        cur_title.clear();
                        cur_link.clear();
                        cur_desc.clear();
                        cur_date.clear();
                    } else if name_lower == "entry" {
                        in_entry = true;
                        cur_title.clear();
                        cur_link.clear();
                        cur_desc.clear();
                        cur_date.clear();
                    } else if in_item || in_entry {
                        current_tag = name_lower.clone();
                        // Atom link with href attribute
                        if (name_lower == "link") && (in_item || in_entry) {
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                if key.eq_ignore_ascii_case("href") {
                                    cur_link = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_item || in_entry {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_tag.as_str() {
                            "title" => cur_title.push_str(&text),
                            "link" => {
                                if cur_link.is_empty() {
                                    cur_link.push_str(&text);
                                }
                            }
                            "description" | "summary" | "content" => cur_desc.push_str(&text),
                            "pubdate" | "published" | "updated" | "dc:date" => cur_date.push_str(&text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::CData(ref e)) => {
                    if in_item || in_entry {
                        let text = String::from_utf8_lossy(e.as_ref()).to_string();
                        match current_tag.as_str() {
                            "title" => cur_title.push_str(&text),
                            "link" => {
                                if cur_link.is_empty() {
                                    cur_link.push_str(&text);
                                }
                            }
                            "description" | "summary" | "content" => cur_desc.push_str(&text),
                            "pubdate" | "published" | "updated" | "dc:date" => cur_date.push_str(&text),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let name_lower = name.to_lowercase();

                    if name_lower == "item" || name_lower == "entry" {
                        in_item = false;
                        in_entry = false;

                        let clean_link = AdBlocker::clean_url_tracking(cur_link.trim());
                        let clean_title = cur_title.trim().to_string();

                        if !clean_title.is_empty() && !clean_link.is_empty() {
                            // Strip HTML tags from summary
                            let plain_summary = Self::strip_html_tags(&cur_desc);
                            let word_count = plain_summary.split_whitespace().count();
                            let reading_time = if word_count == 0 { 1 } else { (word_count + 199) / 200 };

                            let unique_id = format!("{:x}", md5_hash(&format!("{}{}", source_id, clean_link)));

                            items.push(FeedItem {
                                id: unique_id,
                                title: clean_title,
                                link: clean_link,
                                summary: plain_summary,
                                published_date: if cur_date.trim().is_empty() {
                                    None
                                } else {
                                    Some(cur_date.trim().to_string())
                                },
                                source_name: source_name.to_string(),
                                source_id: source_id.to_string(),
                                category: category.to_string(),
                                is_read: false,
                                reading_time_mins: reading_time,
                            });
                        }
                    }
                    current_tag.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    if items.is_empty() {
                        return Err(SecurityError::Network(format!("XML ayrirma hatasi: {}", e)));
                    }
                    break;
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(items)
    }

    fn strip_html_tags(raw: &str) -> String {
        let mut in_tag = false;
        let mut res = String::with_capacity(raw.len());
        for ch in raw.chars() {
            if ch == '<' {
                in_tag = true;
            } else if ch == '>' {
                in_tag = false;
                res.push(' ');
            } else if !in_tag {
                res.push(ch);
            }
        }
        res.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

fn md5_hash(input: &str) -> u64 {
    // Fast 64-bit FNV-1a hash for unique item IDs without external crypto bloat
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
