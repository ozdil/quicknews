use crate::core::adblock::AdBlocker;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanArticle {
    pub title: String,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub source_url: String,
    #[serde(default)]
    pub source_name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub content_text: String,
    pub word_count: usize,
    pub reading_time_mins: usize,
}

pub struct ArticleExtractor;

impl ArticleExtractor {
    /// Extracts clean, ad-free, image-free, distraction-free text from raw HTML.
    pub fn extract(html_str: &str, source_url: &str) -> CleanArticle {
        let document = Html::parse_document(html_str);

        // 1. Extract Title
        let title = Self::extract_title(&document);

        // 2. Extract Author
        let author = Self::extract_meta(&document, "author")
            .or_else(|| Self::extract_meta(&document, "article:author"));

        // 3. Extract Published Date
        let published_date = Self::extract_meta(&document, "article:published_time")
            .or_else(|| Self::extract_meta(&document, "date"))
            .or_else(|| Self::extract_meta(&document, "pubdate"));

        // 4. Extract Main Article Body Content (Text-Only, stripped of any markdown image beacons)
        let raw_body = Self::extract_body_text(&document);
        let content_text = AdBlocker::strip_markdown_images(&raw_body);

        // 5. Calculate statistics
        let word_count = content_text.split_whitespace().count();
        let reading_time_mins = if word_count == 0 {
            1
        } else {
            word_count.div_ceil(200) // standard 200 words per minute
        };

        CleanArticle {
            title,
            author,
            published_date,
            source_url: source_url.to_string(),
            source_name: None,
            category: None,
            tags: Vec::new(),
            content_text,
            word_count,
            reading_time_mins,
        }
    }

    fn extract_title(document: &Html) -> String {
        // Try og:title first
        if let Some(t) = Self::extract_meta(document, "og:title") {
            if !t.trim().is_empty() {
                return t.trim().to_string();
            }
        }

        // Try <h1>
        if let Ok(h1_selector) = Selector::parse("h1") {
            for element in document.select(&h1_selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                let trimmed = text.trim();
                if !trimmed.is_empty() && trimmed.len() > 5 {
                    return trimmed.to_string();
                }
            }
        }

        // Try <title>
        if let Ok(title_selector) = Selector::parse("title") {
            if let Some(element) = document.select(&title_selector).next() {
                let text = element.text().collect::<Vec<_>>().join(" ");
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    // Strip site suffix if present like "Haber Başlığı - Site Adı"
                    if let Some((clean_part, _)) = trimmed.split_once(" - ") {
                        return clean_part.trim().to_string();
                    }
                    if let Some((clean_part, _)) = trimmed.split_once(" | ") {
                        return clean_part.trim().to_string();
                    }
                    return trimmed.to_string();
                }
            }
        }

        "Basliksiz Haber".to_string()
    }

    fn extract_meta(document: &Html, name_or_prop: &str) -> Option<String> {
        let meta_selector = Selector::parse("meta").ok()?;
        for element in document.select(&meta_selector) {
            let name_attr = element
                .value()
                .attr("name")
                .or_else(|| element.value().attr("property"));

            if let Some(name) = name_attr {
                if name.eq_ignore_ascii_case(name_or_prop) {
                    if let Some(content) = element.value().attr("content") {
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_body_text(document: &Html) -> String {
        // Best candidates for article containers
        let candidate_selectors = [
            "[itemprop='articleBody']",
            ".entry-content",
            ".article-content",
            ".article-body",
            ".post-content",
            ".story-body",
            ".news-content",
            ".detail-text",
            ".content-body",
            "article:not(.inloop):not(.related)",
            "article",
            "main",
        ];

        let mut article_element = None;
        for sel_str in &candidate_selectors {
            if let Ok(sel) = Selector::parse(sel_str) {
                if let Some(el) = document.select(&sel).next() {
                    article_element = Some(el);
                    break;
                }
            }
        }

        let mut paragraphs = Vec::new();

        if let Some(root_el) = article_element {
            Self::collect_clean_blocks(&root_el, &mut paragraphs, 0);
        } else if let Ok(body_sel) = Selector::parse("body") {
            if let Some(body_el) = document.select(&body_sel).next() {
                Self::collect_clean_blocks(&body_el, &mut paragraphs, 0);
            }
        }

        // Deduplicate and filter out trivial short lines, cap total blocks and size
        let mut final_blocks = Vec::new();
        let mut total_chars: usize = 0;
        for p in paragraphs {
            if final_blocks.len() >= 300 || total_chars >= 150_000 {
                break;
            }
            let trimmed = p.trim();
            if trimmed.len() >= 25 {
                // Ensure no duplicates
                if !final_blocks.contains(&trimmed.to_string()) {
                    total_chars += trimmed.len();
                    final_blocks.push(trimmed.to_string());
                }
            }
        }

        if final_blocks.is_empty() {
            "Haber metni ayrilamadi veya sayfa saf metin icermiyor.".to_string()
        } else {
            final_blocks.join("\n\n")
        }
    }

    fn collect_clean_blocks(element: &scraper::ElementRef, out: &mut Vec<String>, depth: usize) {
        const MAX_DOM_DEPTH: usize = 32;
        const MAX_BLOCKS_LIMIT: usize = 500;
        if depth > MAX_DOM_DEPTH || out.len() >= MAX_BLOCKS_LIMIT {
            return;
        }

        let tag_name = element.value().name();

        // Strictly reject forbidden tags (scripts, ads, images, etc.)
        if AdBlocker::is_forbidden_tag(tag_name) {
            return;
        }

        // Check classes and IDs for ads and clutter
        if let Some(class) = element.value().attr("class") {
            if AdBlocker::is_ad_or_clutter_attribute(class) {
                return;
            }
        }
        if let Some(id) = element.value().attr("id") {
            if AdBlocker::is_ad_or_clutter_attribute(id) {
                return;
            }
        }

        if matches!(tag_name, "h1" | "h2" | "h3" | "h4") {
            let text = element.text().collect::<Vec<_>>().join(" ");
            let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !cleaned.is_empty() {
                out.push(format!("## {}", cleaned));
            }
            return;
        }

        if tag_name == "blockquote" {
            let text = element.text().collect::<Vec<_>>().join(" ");
            let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !cleaned.is_empty() {
                out.push(format!("> {}", cleaned));
            }
            return;
        }

        if tag_name == "li" {
            let is_in_ol = element.parent().and_then(scraper::ElementRef::wrap).is_some_and(|p| p.value().name() == "ol");
            let text = element.text().collect::<Vec<_>>().join(" ");
            let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !cleaned.is_empty() {
                if is_in_ol {
                    out.push(format!("1. {}", cleaned));
                } else {
                    out.push(format!("- {}", cleaned));
                }
            }
            return;
        }

        if tag_name == "p" {
            let text = element.text().collect::<Vec<_>>().join(" ");
            let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !cleaned.is_empty() {
                out.push(cleaned);
            }
            return;
        }

        // For other container elements (div, article, section, etc.):
        // Walk children. If children contain text nodes and <br>, accumulate them into paragraphs!
        let mut current_inline = String::new();
        for child in element.children() {
            if let Some(text_node) = child.value().as_text() {
                let t = text_node.trim();
                if !t.is_empty() {
                    if !current_inline.is_empty() && !current_inline.ends_with(' ') {
                        current_inline.push(' ');
                    }
                    current_inline.push_str(t);
                }
            } else if let Some(child_el) = scraper::ElementRef::wrap(child) {
                let child_tag = child_el.value().name();
                if child_tag == "br" {
                    let cleaned = current_inline.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !cleaned.is_empty() {
                        out.push(cleaned);
                        current_inline.clear();
                    }
                } else if matches!(child_tag, "b" | "strong" | "i" | "em" | "a" | "code" | "span") {
                    let t = child_el.text().collect::<Vec<_>>().join(" ");
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        if !current_inline.is_empty() && !current_inline.ends_with(' ') {
                            current_inline.push(' ');
                        }
                        current_inline.push_str(trimmed);
                    }
                } else {
                    // Flush accumulated inline text before recursing into block child
                    let cleaned = current_inline.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !cleaned.is_empty() {
                        out.push(cleaned);
                        current_inline.clear();
                    }
                    Self::collect_clean_blocks(&child_el, out, depth + 1);
                }
            }
        }

        let cleaned = current_inline.split_whitespace().collect::<Vec<_>>().join(" ");
        if !cleaned.is_empty() {
            out.push(cleaned);
        }
    }

    /// Detects pagination URLs for multi-page articles (e.g. Phoronix, reviews, paginated reports).
    /// Enforces strict same-origin, same-base-path checks and caps at 10 pages maximum.
    pub fn detect_pagination_urls(html_str: &str, base_url: &str) -> Vec<String> {
        let base_parsed = match url::Url::parse(base_url) {
            Ok(u) => u,
            Err(_) => return Vec::new(),
        };

        let document = Html::parse_document(html_str);
        let mut candidate_urls = Vec::new();

        // 1. Selector options (e.g. Phoronix <select id="phx_article_page_selector"> <option value="...">)
        if let Ok(sel) = Selector::parse("select#phx_article_page_selector option, select.pagination option") {
            for el in document.select(&sel) {
                if let Some(val) = el.value().attr("value") {
                    if let Ok(resolved) = base_parsed.join(val) {
                        candidate_urls.push(resolved);
                    }
                }
            }
        }

        // 2. Pagination containers: .pagination, .page-numbers, nav.pagination, .pager
        let pagination_selectors = [
            ".pagination a",
            ".page-numbers a",
            "nav.pagination a",
            ".pager a",
            ".pages a",
            "a[rel='next']",
            "link[rel='next']",
        ];

        for sel_str in &pagination_selectors {
            if let Ok(sel) = Selector::parse(sel_str) {
                for el in document.select(&sel) {
                    let href_attr = el.value().attr("href");
                    if let Some(href) = href_attr {
                        if let Ok(resolved) = base_parsed.join(href) {
                            candidate_urls.push(resolved);
                        }
                    }
                }
            }
        }

        // 3. Fallback heuristic: "Page 1 of X" or "Sayfa 1 / X" in text
        let raw_text = document.root_element().text().collect::<Vec<_>>().join(" ");
        if let Some(total_pages) = Self::detect_total_pages_from_text(&raw_text) {
            if total_pages > 1 && total_pages <= 15 {
                for page_num in 2..=total_pages {
                    let clean_base = base_url.trim_end_matches('/');
                    if clean_base.contains("/review/") || clean_base.contains("/article/") || clean_base.contains("/haber/") {
                        let candidate_url = format!("{}/{}", clean_base, page_num);
                        if let Ok(resolved) = url::Url::parse(&candidate_url) {
                            candidate_urls.push(resolved);
                        }
                    }
                }
            }
        }

        // Filter and deduplicate candidates
        let mut final_urls: Vec<String> = Vec::new();
        let base_normalized = {
            let mut b = base_parsed.clone();
            b.set_fragment(None);
            b.to_string().trim_end_matches('/').to_string()
        };

        for u in candidate_urls {
            let mut norm_u = u.clone();
            norm_u.set_fragment(None);

            // Same origin check
            if norm_u.origin() != base_parsed.origin() {
                continue;
            }

            let u_str = norm_u.to_string();
            let u_trimmed = u_str.trim_end_matches('/').to_string();

            // Ignore current base page
            if u_trimmed == base_normalized || u_trimmed == format!("{}/1", base_normalized) {
                continue;
            }

            // Must be related to the base path (e.g. starts with same base path prefix)
            let base_path = base_parsed.path().trim_end_matches('/');
            if (!norm_u.path().starts_with(base_path) && !norm_u.path().contains(base_path))
                && norm_u.path() != base_parsed.path()
            {
                continue;
            }

            if !final_urls.contains(&u_str) {
                final_urls.push(u_str);
            }
        }

        // Cap at 10 pages maximum
        final_urls.truncate(10);
        final_urls
    }

    fn detect_total_pages_from_text(text: &str) -> Option<usize> {
        let lower = text.to_lowercase();
        let patterns = ["page 1 of ", "page 1 / ", "sayfa 1 / ", "sayfa 1 of "];
        for pat in &patterns {
            if let Some(pos) = lower.find(pat) {
                let after = &lower[pos + pat.len()..];
                let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(num) = num_str.parse::<usize>() {
                    return Some(num);
                }
            }
        }
        None
    }
}
