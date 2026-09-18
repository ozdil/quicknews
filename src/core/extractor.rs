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

        // 4. Extract Main Article Body Content (Text-Only)
        let content_text = Self::extract_body_text(&document);

        // 5. Calculate statistics
        let word_count = content_text.split_whitespace().count();
        let reading_time_mins = if word_count == 0 {
            1
        } else {
            (word_count + 199) / 200 // standard 200 words per minute
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
            Self::collect_clean_blocks(&root_el, &mut paragraphs);
        } else if let Ok(body_sel) = Selector::parse("body") {
            if let Some(body_el) = document.select(&body_sel).next() {
                Self::collect_clean_blocks(&body_el, &mut paragraphs);
            }
        }

        // Deduplicate and filter out trivial short lines
        let mut final_blocks = Vec::new();
        for p in paragraphs {
            let trimmed = p.trim();
            if trimmed.len() >= 25 {
                // Ensure no duplicates
                if !final_blocks.contains(&trimmed.to_string()) {
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

    fn collect_clean_blocks(element: &scraper::ElementRef, out: &mut Vec<String>) {
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
            let is_in_ol = element.parent().and_then(scraper::ElementRef::wrap).map_or(false, |p| p.value().name() == "ol");
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
                    Self::collect_clean_blocks(&child_el, out);
                }
            }
        }

        let cleaned = current_inline.split_whitespace().collect::<Vec<_>>().join(" ");
        if !cleaned.is_empty() {
            out.push(cleaned);
        }
    }
}
