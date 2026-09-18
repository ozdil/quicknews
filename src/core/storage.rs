use crate::core::extractor::CleanArticle;
use crate::core::feed::FeedItem;
use crate::core::security::{atomic_write_file, safe_read_file, SecurityError, MAX_LOCAL_FILE_SIZE};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub feed_url: String,
    pub category: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppData {
    pub sources: Vec<SourceConfig>,
    pub articles: Vec<FeedItem>,
}

pub struct StorageManager {
    config_dir: PathBuf,
    data_dir: PathBuf,
}

impl StorageManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_dir = PathBuf::from(&home).join(".config").join("quicknews");
        let data_dir = PathBuf::from(&home).join(".local").join("share").join("quicknews");

        Self {
            config_dir,
            data_dir,
        }
    }

    pub fn sources_path(&self) -> PathBuf {
        self.config_dir.join("sources.json")
    }

    pub fn articles_path(&self) -> PathBuf {
        self.data_dir.join("articles.json")
    }

    pub fn saved_path(&self) -> PathBuf {
        self.data_dir.join("saved.json")
    }

    pub fn content_cache_path(&self, url: &str) -> PathBuf {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in url.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.data_dir
            .join("content_cache")
            .join(format!("{:x}.json", hash))
    }

    pub fn load_sources(&self) -> Result<Vec<SourceConfig>, SecurityError> {
        let p = self.sources_path();
        if !p.exists() {
            // If sources.json doesn't exist, create default sources
            let defaults = Self::default_sources();
            self.save_sources(&defaults)?;
            return Ok(defaults);
        }

        let bytes = safe_read_file(&p, MAX_LOCAL_FILE_SIZE)?;
        let sources = serde_json::from_slice(&bytes)
            .map_err(|e| SecurityError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        Ok(sources)
    }

    pub fn save_sources(&self, sources: &[SourceConfig]) -> Result<(), SecurityError> {
        let json = serde_json::to_vec_pretty(sources)
            .map_err(|e| SecurityError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        atomic_write_file(&self.sources_path(), &json)
    }

    pub fn add_source(
        &self,
        name: &str,
        domain: &str,
        feed_url: &str,
        category: &str,
    ) -> Result<bool, SecurityError> {
        let mut sources = self.load_sources().unwrap_or_default();

        // Prevent duplicates by feed_url or domain
        let f_low = feed_url.to_lowercase();
        let d_low = domain.to_lowercase();
        if sources.iter().any(|s| s.feed_url.to_lowercase() == f_low || s.domain.to_lowercase() == d_low) {
            return Ok(false); // Already exists
        }

        let id = format!("src_{:x}", sources.len() + 1);
        sources.push(SourceConfig {
            id,
            name: name.to_string(),
            domain: domain.to_string(),
            feed_url: feed_url.to_string(),
            category: category.to_string(),
            enabled: true,
        });

        self.save_sources(&sources)?;
        Ok(true)
    }

    pub fn remove_source(&self, id_or_domain: &str) -> Result<bool, SecurityError> {
        let mut sources = self.load_sources().unwrap_or_default();
        let orig_len = sources.len();
        sources.retain(|s| s.id != id_or_domain && !s.domain.eq_ignore_ascii_case(id_or_domain));

        if sources.len() != orig_len {
            self.save_sources(&sources)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn load_articles(&self) -> Vec<FeedItem> {
        let p = self.articles_path();
        if !p.exists() {
            return Vec::new();
        }

        if let Ok(bytes) = safe_read_file(&p, MAX_LOCAL_FILE_SIZE) {
            if let Ok(items) = serde_json::from_slice::<Vec<FeedItem>>(&bytes) {
                return items;
            }
        }
        Vec::new()
    }

    pub fn save_articles(&self, articles: &[FeedItem]) -> Result<(), SecurityError> {
        let json = serde_json::to_vec_pretty(articles)
            .map_err(|e| SecurityError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        atomic_write_file(&self.articles_path(), &json)
    }

    pub fn mark_article_read(&self, article_id: &str) -> Result<(), SecurityError> {
        let mut articles = self.load_articles();
        for item in &mut articles {
            if item.id == article_id {
                item.is_read = true;
            }
        }
        self.save_articles(&articles)
    }

    pub fn get_cached_content(&self, url: &str) -> Option<CleanArticle> {
        let p = self.content_cache_path(url);
        if let Ok(bytes) = safe_read_file(&p, MAX_LOCAL_FILE_SIZE) {
            if let Ok(art) = serde_json::from_slice::<CleanArticle>(&bytes) {
                return Some(art);
            }
        }
        None
    }

    pub fn cache_article_content(&self, url: &str, article: &CleanArticle) -> Result<(), SecurityError> {
        let p = self.content_cache_path(url);
        let json = serde_json::to_vec_pretty(article)
            .map_err(|e| SecurityError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        atomic_write_file(&p, &json)
    }

    pub fn load_saved(&self) -> Vec<FeedItem> {
        let p = self.saved_path();
        if !p.exists() {
            return Vec::new();
        }
        if let Ok(bytes) = safe_read_file(&p, MAX_LOCAL_FILE_SIZE) {
            if let Ok(items) = serde_json::from_slice::<Vec<FeedItem>>(&bytes) {
                return items;
            }
        }
        Vec::new()
    }

    pub fn save_article_bookmark(&self, item: &FeedItem) -> Result<bool, SecurityError> {
        let mut saved = self.load_saved();
        if saved.iter().any(|s| s.id == item.id || s.link == item.link) {
            return Ok(false); // Already saved
        }
        let mut new_item = item.clone();
        new_item.is_read = true;
        saved.insert(0, new_item);
        let json = serde_json::to_vec_pretty(&saved)
            .map_err(|e| SecurityError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        atomic_write_file(&self.saved_path(), &json)?;
        Ok(true)
    }

    pub fn remove_saved_article(&self, id_or_link: &str) -> Result<bool, SecurityError> {
        let mut saved = self.load_saved();
        let orig_len = saved.len();
        saved.retain(|s| s.id != id_or_link && s.link != id_or_link);
        if saved.len() != orig_len {
            let json = serde_json::to_vec_pretty(&saved)
                .map_err(|e| SecurityError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
            atomic_write_file(&self.saved_path(), &json)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn is_article_saved(&self, id_or_link: &str) -> bool {
        let saved = self.load_saved();
        saved.iter().any(|s| s.id == id_or_link || s.link == id_or_link)
    }

    pub fn export_article_markdown(
        &self,
        article: &CleanArticle,
        target_path: Option<&std::path::Path>,
    ) -> Result<PathBuf, SecurityError> {
        let dest = if let Some(p) = target_path {
            p.to_path_buf()
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let export_dir = PathBuf::from(&home).join("Belgeler").join("QuickNews");
            let safe_title: String = article
                .title
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
                .take(60)
                .collect();
            export_dir.join(format!("{}.md", safe_title))
        };

        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", article.title));
        if let Some(ref auth) = article.author {
            md.push_str(&format!("**Yazar:** {}\n\n", auth));
        }
        if let Some(ref date) = article.published_date {
            md.push_str(&format!("**Tarih:** {}\n\n", date));
        }
        md.push_str(&format!("**Kaynak:** {}\n\n", article.source_url));
        md.push_str(&format!("**Okuma Suresi:** ~{} dakika\n\n", article.reading_time_mins));
        md.push_str("---\n\n");
        md.push_str(&article.content_text);
        md.push('\n');

        atomic_write_file(&dest, md.as_bytes())?;
        Ok(dest)
    }

    fn default_sources() -> Vec<SourceConfig> {
        vec![
            SourceConfig {
                id: "src_1".to_string(),
                name: "Webrazzi".to_string(),
                domain: "webrazzi.com".to_string(),
                feed_url: "https://webrazzi.com/feed/".to_string(),
                category: "Teknoloji".to_string(),
                enabled: true,
            },
            SourceConfig {
                id: "src_2".to_string(),
                name: "ShiftDelete".to_string(),
                domain: "shiftdelete.net".to_string(),
                feed_url: "https://shiftdelete.net/feed".to_string(),
                category: "Teknoloji".to_string(),
                enabled: true,
            },
            SourceConfig {
                id: "src_3".to_string(),
                name: "Phoronix".to_string(),
                domain: "phoronix.com".to_string(),
                feed_url: "https://www.phoronix.com/rss.php".to_string(),
                category: "Linux & Donanim".to_string(),
                enabled: true,
            },
            SourceConfig {
                id: "src_4".to_string(),
                name: "Evrim Agaci".to_string(),
                domain: "evrimagaci.org".to_string(),
                feed_url: "https://evrimagaci.org/rss.xml".to_string(),
                category: "Bilim".to_string(),
                enabled: true,
            },
        ]
    }
}
