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

        // Auto classify tags if empty
        for item in &mut all_articles {
            if item.tags.is_empty() {
                item.tags = AiEngine::auto_classify_tags(&item.source_name, &item.category, &item.title, &item.summary);
            }
        }

        // Cap total cached articles to 500 to keep memory small and snappy
        if all_articles.len() > 500 {
            all_articles.truncate(500);
        }

        let _ = self.storage.save_articles(&all_articles);
        all_articles
    }

    /// Fetches and cleans full article text (ad-free, image-free, AI structured).
    pub async fn read_clean_article(&self, url: &str) -> Result<CleanArticle, SecurityError> {
        // Check cache first
        if let Some(cached) = self.storage.get_cached_content(url) {
            return Ok(cached);
        }

        // Fetch raw HTML (up to 2 MiB, SSRF guarded)
        let html_content = fetch_bounded_content(url, MAX_HTTP_PAYLOAD_SIZE, 8).await?;
        let mut article = ArticleExtractor::extract(&html_content, url);

        // AI processes 100% of the article text: structures paragraphs, preserves lists, strips clutter
        let ai_structured_text = AiEngine::clean_full_article_content(&article.title, &article.content_text).await;
        article.content_text = ai_structured_text;
        let word_count = article.content_text.split_whitespace().count();
        article.word_count = word_count;
        article.reading_time_mins = if word_count == 0 { 1 } else { (word_count + 199) / 200 };

        // Attach source metadata and auto-classified tags
        let existing_items = self.storage.load_articles();
        if let Some(item) = existing_items.iter().find(|i| i.link == url) {
            article.source_name = Some(item.source_name.clone());
            article.category = Some(item.category.clone());
            article.tags = item.tags.clone();
        }
        if article.tags.is_empty() {
            let s_name = article.source_name.as_deref().unwrap_or("Teknoloji");
            let c_name = article.category.as_deref().unwrap_or("Teknoloji");
            article.tags = AiEngine::auto_classify_tags(s_name, c_name, &article.title, &article.content_text);
        }

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

    /// Toggles bookmark / saved status of an article by ID or URL.
    pub fn toggle_save_article(&self, id_or_link: &str) -> (bool, bool) {
        if self.storage.is_article_saved(id_or_link) {
            let ok = self.storage.remove_saved_article(id_or_link).unwrap_or(false);
            (false, ok)
        } else {
            let articles = self.storage.load_articles();
            if let Some(item) = articles.iter().find(|a| a.id == id_or_link || a.link == id_or_link) {
                let ok = self.storage.save_article_bookmark(item).unwrap_or(false);
                (true, ok)
            } else {
                (false, false)
            }
        }
    }

    /// Loads all bookmarked / saved articles.
    pub fn load_saved_articles(&self) -> Vec<FeedItem> {
        self.storage.load_saved()
    }

    /// Exports full clean article to Markdown file.
    pub async fn export_article_to_markdown(
        &self,
        url: &str,
        target_path: Option<&std::path::Path>,
    ) -> Result<std::path::PathBuf, SecurityError> {
        let clean = self.read_clean_article(url).await?;
        self.storage.export_article_markdown(&clean, target_path)
    }
}
