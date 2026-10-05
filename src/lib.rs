pub mod core;

use crate::core::ai::{AiEngine, DiscoveredSource};
use crate::core::extractor::{ArticleExtractor, CleanArticle};
use crate::core::feed::{FeedItem, FeedParser};
use crate::core::security::{fetch_bounded_content, SecurityError, MAX_HTTP_PAYLOAD_SIZE};
use crate::core::storage::StorageManager;

pub struct QuickNewsApp {
    pub storage: StorageManager,
}

impl Default for QuickNewsApp {
    fn default() -> Self {
        Self::new()
    }
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
            let s_name = src.name.clone();
            tasks.push(tokio::spawn(async move {
                let res = FeedParser::fetch_feed_items(&src.feed_url, &src.id, &src.name, &src.category).await;
                (s_name, res)
            }));
        }

        let existing = self.storage.load_articles();
        let mut existing_ids: std::collections::HashSet<String> =
            existing.iter().map(|a| a.id.clone()).collect();

        let mut new_items = Vec::new();
        for task in tasks {
            if let Ok((name, res)) = task.await {
                match res {
                    Ok(items) => {
                        let count = items.len();
                        for item in items {
                            if !existing_ids.contains(&item.id) {
                                existing_ids.insert(item.id.clone());
                                new_items.push(item);
                            }
                        }
                        if count > 0 {
                            eprintln!("[QuickNews] '{}' kaynagindan {} haber alindi.", name, count);
                        }
                    }
                    Err(e) => {
                        eprintln!("[QuickNews] '{}' kaynagi cekilemedi: {}", name, e);
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

        // Cap total cached articles to 1500 to keep memory small and snappy
        if all_articles.len() > 1500 {
            all_articles.truncate(1500);
        }

        let _ = self.storage.save_articles(&all_articles);
        all_articles
    }

    /// Fetches and cleans full article text (ad-free, image-free, multi-page stitched, AI structured).
    pub async fn read_clean_article(&self, url: &str) -> Result<CleanArticle, SecurityError> {
        // Check cache first (invalidate if stale pagination remnant exists)
        if let Some(cached) = self.storage.get_cached_content(url) {
            let lower = cached.content_text.to_lowercase();
            let has_pagination_remnant = (lower.contains("page 1 of")
                || lower.contains("page: 1 2")
                || lower.contains("page, 1, 2")
                || lower.contains("page 1 2 3")
                || lower.contains("sayfa 1 /")
                || lower.contains("sayfa: 1 2"))
                && !cached.content_text.contains("## Sayfa 2");

            if !has_pagination_remnant {
                return Ok(cached);
            }
        }

        // Fetch raw HTML of the first page (up to 2 MiB, SSRF guarded)
        let html_content = fetch_bounded_content(url, MAX_HTTP_PAYLOAD_SIZE, 8).await?;
        let mut article = ArticleExtractor::extract(&html_content, url);

        // Check for multi-page pagination (e.g. Phoronix, reviews, multi-page articles)
        let pagination_urls = ArticleExtractor::detect_pagination_urls(&html_content, url);
        if !pagination_urls.is_empty() {
            let mut page_index = 2;
            let norm_title = article.title.trim().to_lowercase();
            for page_url in pagination_urls {
                if let Ok(page_html) = fetch_bounded_content(&page_url, MAX_HTTP_PAYLOAD_SIZE, 8).await {
                    let page_article = ArticleExtractor::extract(&page_html, &page_url);
                    let mut clean_page_text = page_article.content_text.trim();

                    // Strip duplicated main title from the top of the subpage
                    let title_with_h2 = format!("## {}", article.title.trim());
                    if clean_page_text.starts_with(&title_with_h2) {
                        clean_page_text = clean_page_text[title_with_h2.len()..].trim();
                    } else if clean_page_text.starts_with(article.title.trim()) {
                        clean_page_text = clean_page_text[article.title.trim().len()..].trim();
                    } else if clean_page_text.to_lowercase().starts_with(&norm_title) {
                        clean_page_text = clean_page_text[norm_title.len()..].trim();
                    }

                    if clean_page_text.len() > 30 && !clean_page_text.starts_with("Haber metni ayrilamadi") {
                        article.content_text.push_str(&format!("\n\n\n## Sayfa {}\n\n{}", page_index, clean_page_text));
                        page_index += 1;
                    }
                }
            }
        }

        // AI processes 100% of the stitched article text: structures paragraphs, preserves lists, strips clutter
        let ai_structured_text = AiEngine::clean_full_article_content(&article.title, &article.content_text).await;
        article.content_text = crate::core::adblock::AdBlocker::strip_markdown_images(&ai_structured_text);
        let word_count = article.content_text.split_whitespace().count();
        article.word_count = word_count;
        article.reading_time_mins = if word_count == 0 { 1 } else { word_count.div_ceil(200) };

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

    /// Adds a source only if it provides a functional RSS/Atom XML feed.
    /// Rejects dead/non-RSS links and probes for working alternative feeds before giving up.
    pub async fn add_source_verified(
        &self,
        name: &str,
        domain: &str,
        feed_url: &str,
        category: &str,
    ) -> Result<bool, SecurityError> {
        match FeedParser::verify_feed_endpoint(feed_url).await {
            Ok(_) => self.storage.add_source(name, domain, feed_url, category),
            Err(orig_err) => {
                let site_target = format!("https://{}", domain);
                if let Ok(discovered) = FeedParser::discover_feed_url(&site_target).await {
                    if FeedParser::verify_feed_endpoint(&discovered).await.is_ok() {
                        eprintln!(
                            "[QuickNews] '{}' adresi gecersizdi, ancak guncel XML beslemesi bulundu: '{}'",
                            feed_url, discovered
                        );
                        return self.storage.add_source(name, domain, &discovered, category);
                    }
                }
                Err(SecurityError::Network(format!(
                    "Guncel RSS/Atom XML teknolojisi sunmayan kaynak eklenmedi: {}",
                    orig_err
                )))
            }
        }
    }

    /// Natural language source discovery and auto-addition with strict XML feed verification.
    pub async fn discover_and_add_sources(&self, prompt: &str) -> Vec<DiscoveredSource> {
        let candidates = AiEngine::discover_sources_from_prompt(prompt).await;
        let mut added = Vec::new();

        let existing = self.storage.load_sources().unwrap_or_default();
        let mut seen_domains: std::collections::HashSet<String> = existing
            .iter()
            .map(|s| s.domain.trim().to_lowercase())
            .collect();
        let mut seen_feeds: std::collections::HashSet<String> = existing
            .iter()
            .map(|s| s.feed_url.trim().to_lowercase())
            .collect();

        for mut candidate in candidates {
            let d_clean = candidate.domain.trim().to_lowercase();
            if seen_domains.contains(&d_clean) {
                // Kaynak zaten mevcut listede bulunuyor, yeni adaylara odaklan
                continue;
            }

            let initial_feed = candidate.suggested_feed.clone();

            let valid_feed = if let Some(ref sf) = initial_feed {
                let sf_clean = sf.trim().to_lowercase();
                if seen_feeds.contains(&sf_clean) {
                    continue;
                }

                if FeedParser::verify_feed_endpoint(sf).await.is_ok() {
                    Some(sf.clone())
                } else {
                    let site_target = format!("https://{}", candidate.domain);
                    match FeedParser::discover_feed_url(&site_target).await {
                        Ok(discovered) if FeedParser::verify_feed_endpoint(&discovered).await.is_ok() => Some(discovered),
                        _ => None,
                    }
                }
            } else {
                let site_target = format!("https://{}", candidate.domain);
                match FeedParser::discover_feed_url(&site_target).await {
                    Ok(discovered) if FeedParser::verify_feed_endpoint(&discovered).await.is_ok() => Some(discovered),
                    _ => None,
                }
            };

            // If no working RSS/Atom XML technology found, DO NOT ADD ("ekleme yapmasin")
            let verified_url = match valid_feed {
                Some(url) => url,
                None => {
                    eprintln!(
                        "[QuickNews] '{}' ({}) guncel bir RSS/Atom XML akisi sunmadigi icin eklenmedi.",
                        candidate.name, candidate.domain
                    );
                    continue;
                }
            };

            candidate.suggested_feed = Some(verified_url.clone());

            // Add source to storage
            if let Ok(true) = self.storage.add_source(
                &candidate.name,
                &candidate.domain,
                &verified_url,
                &candidate.category,
            ) {
                seen_domains.insert(d_clean);
                seen_feeds.insert(verified_url.trim().to_lowercase());
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

    /// Toggles read / unread status of an article by ID or URL.
    pub fn toggle_read_article(&self, id_or_link: &str) -> (bool, bool) {
        self.storage.toggle_article_read(id_or_link)
    }

    /// Sets read status of an article by ID or URL.
    pub fn set_article_read(&self, id_or_link: &str, is_read: bool) -> Result<(), SecurityError> {
        self.storage.set_article_read_state(id_or_link, is_read)
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
