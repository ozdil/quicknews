pub mod core;

use crate::core::ai::{AiEngine, DiscoveredSource};
use crate::core::extractor::{ArticleExtractor, CleanArticle};
use crate::core::feed::{FeedItem, FeedParser};
use crate::core::security::{fetch_bounded_content, SecurityError, MAX_HTTP_PAYLOAD_SIZE};
use crate::core::storage::StorageManager;

pub struct QuickNewsApp {
    pub storage: StorageManager,
}

impl QuickNewsApp {
    pub fn new() -> Self {
        Self {
            storage: StorageManager::new(),
        }
    }

    /// Fetches all feeds concurrently from enabled sources.
    pub async fn sync_all_feeds(&self) -> Vec<FeedItem> {
        let sources = self.storage.load_sources().unwrap_or_default();
        let mut tasks = Vec::new();

        for src in sources.into_iter().filter(|s| s.enabled) {
            tasks.push(tokio::spawn(async move {
                FeedParser::fetch_feed_items(&src.feed_url, &src.id, &src.name, &src.category).await
            }));
        }

        let existing = self.storage.load_articles();
        let mut existing_ids: std::collections::HashSet<String> =
            existing.iter().map(|a| a.id.clone()).collect();

        let mut new_items = Vec::new();
        for task in tasks {
            if let Ok(Ok(items)) = task.await {
                for item in items {
                    if !existing_ids.contains(&item.id) {
                        existing_ids.insert(item.id.clone());
                        new_items.push(item);
                    }
                }
            }
        }

        // Prepend new items to keep chronologically fresh articles on top
        let mut all_articles = new_items;
        all_articles.extend(existing);

        // Cap total cached articles to 200 to keep memory small and snappy
        if all_articles.len() > 200 {
            all_articles.truncate(200);
        }

        let _ = self.storage.save_articles(&all_articles);
        all_articles
    }

    /// Fetches and cleans full article text (ad-free, image-free).
    pub async fn read_clean_article(&self, url: &str) -> Result<CleanArticle, SecurityError> {
        // Check cache first
        if let Some(cached) = self.storage.get_cached_content(url) {
            return Ok(cached);
        }

        // Fetch raw HTML (up to 2 MiB, SSRF guarded)
        let html_content = fetch_bounded_content(url, MAX_HTTP_PAYLOAD_SIZE, 8).await?;
        let article = ArticleExtractor::extract(&html_content, url);

        // Cache clean result
        let _ = self.storage.cache_article_content(url, &article);
        Ok(article)
    }

    /// Natural language source discovery and auto-addition.
    pub async fn discover_and_add_sources(&self, prompt: &str) -> Vec<DiscoveredSource> {
        let candidates = AiEngine::discover_sources_from_prompt(prompt).await;
        let mut added = Vec::new();

        for mut candidate in candidates {
            // If suggested feed is missing or unverified, run auto-discovery
            let feed_url = if let Some(ref sf) = candidate.suggested_feed {
                sf.clone()
            } else {
                let site_target = format!("https://{}", candidate.domain);
                match FeedParser::discover_feed_url(&site_target).await {
                    Ok(url) => url,
                    Err(_) => continue,
                }
            };

            candidate.suggested_feed = Some(feed_url.clone());

            // Add source to storage
            if let Ok(true) = self.storage.add_source(
                &candidate.name,
                &candidate.domain,
                &feed_url,
                &candidate.category,
            ) {
                added.push(candidate);
            }
        }

        added
    }
}
