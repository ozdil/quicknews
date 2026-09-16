use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredSource {
    pub name: String,
    pub domain: String,
    pub suggested_feed: Option<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSummaryResult {
    pub neutral_title: String,
    pub key_points: Vec<String>,
    pub reading_time_mins: usize,
}

pub struct AiEngine;

impl AiEngine {
    /// Discovers news sources from natural language prompt (e.g. "Turkiye teknoloji sitelerinden 10 tanesini ekle")
    pub async fn discover_sources_from_prompt(prompt: &str) -> Vec<DiscoveredSource> {
        let p_lower = prompt.to_lowercase();

        // 1. Try Google Gemini API if GEMINI_API_KEY is present
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            if !api_key.trim().is_empty() {
                if let Ok(res) = Self::query_gemini_for_sources(prompt, &api_key).await {
                    if !res.is_empty() {
                        return res;
                    }
                }
            }
        }

        // 2. Try Local Ollama if running
        if let Ok(res) = Self::query_ollama_for_sources(prompt).await {
            if !res.is_empty() {
                return res;
            }
        }

        // 3. Built-in Curated Intelligence Knowledge Base (Instant, Offline & Zero-API Fallback)
        Self::resolve_curated_knowledge_base(&p_lower)
    }

    /// Generates neutral headline and 3 key points summary from clean article text.
    pub async fn summarize_article(
        title: &str,
        content_text: &str,
    ) -> AiSummaryResult {
        // Try Gemini if available
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            if !api_key.trim().is_empty() {
                if let Ok(res) = Self::query_gemini_summary(title, content_text, &api_key).await {
                    return res;
                }
            }
        }

        // Try Ollama if available
        if let Ok(res) = Self::query_ollama_summary(title, content_text).await {
            return res;
        }

        // Fast & deterministic Local Extractive NLP fallback
        Self::local_extractive_summary(title, content_text)
    }

    /// Local extractive NLP summarizer: extracts the top 3 most informative sentences.
    pub fn local_extractive_summary(title: &str, content: &str) -> AiSummaryResult {
        let word_count = content.split_whitespace().count();
        let reading_time = if word_count == 0 { 1 } else { (word_count + 199) / 200 };

        // Neutralize title: remove exclamation marks, clickbait prefixes
        let mut clean_title = title.trim().to_string();
        let clickbait_words = ["FLAS!", "SON DAKIKA:", "INANILMAZ!", "SOK!", "DIKKAT!", "MUDE!", "HERKESI SASIRTTI!"];
        for cw in &clickbait_words {
            if clean_title.to_uppercase().starts_with(cw) {
                clean_title = clean_title[cw.len()..].trim().to_string();
            }
        }

        // Split text into candidate sentences
        let sentences: Vec<&str> = content
            .split(&['.', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| s.len() >= 35 && s.len() <= 220)
            .collect();

        if sentences.is_empty() {
            return AiSummaryResult {
                neutral_title: clean_title,
                key_points: vec![
                    "Icerik cok kisa veya metin ayiklanamadi.".to_string(),
                ],
                reading_time_mins: reading_time,
            };
        }

        // Compute term frequencies (stopwords excluded)
        let mut freq_map: HashMap<String, usize> = HashMap::new();
        for word in content.split_whitespace() {
            let w = word.to_lowercase();
            if w.len() > 3 {
                *freq_map.entry(w).or_insert(0) += 1;
            }
        }

        // Score sentences
        let mut scored_sentences: Vec<(usize, &str)> = sentences
            .iter()
            .enumerate()
            .map(|(idx, sentence)| {
                let mut score: usize = 0;
                for w in sentence.split_whitespace() {
                    let w_low = w.to_lowercase();
                    if let Some(&cnt) = freq_map.get(&w_low) {
                        score += cnt;
                    }
                }
                // First 3 sentences get positional boost
                if idx < 3 {
                    score += 10;
                }
                (score, *sentence)
            })
            .collect();

        // Sort descending by score
        scored_sentences.sort_by(|a, b| b.0.cmp(&a.0));

        let top_count = std::cmp::min(3, scored_sentences.len());
        let mut bullets = Vec::new();
        for i in 0..top_count {
            bullets.push(format!("{}.", scored_sentences[i].1));
        }

        AiSummaryResult {
            neutral_title: clean_title,
            key_points: bullets,
            reading_time_mins: reading_time,
        }
    }

    fn resolve_curated_knowledge_base(query: &str) -> Vec<DiscoveredSource> {
        let mut results = Vec::new();

        let is_tr = query.contains("turkiye") || query.contains("türkiye") || query.contains("turk") || query.contains("türk");
        let is_tech = query.contains("teknoloji") || query.contains("yazilim") || query.contains("yazılım") || query.contains("bilisim") || query.contains("tech");
        let is_linux = query.contains("linux") || query.contains("acik kaynak") || query.contains("açık kaynak") || query.contains("open source");
        let is_science = query.contains("bilim") || query.contains("uzay") || query.contains("science");
        let is_finance = query.contains("ekonomi") || query.contains("finans") || query.contains("borsa") || query.contains("dolar");

        if is_tr && is_tech {
            results.push(DiscoveredSource {
                name: "Webrazzi".to_string(),
                domain: "webrazzi.com".to_string(),
                suggested_feed: Some("https://webrazzi.com/feed/".to_string()),
                category: "Teknoloji".to_string(),
            });
            results.push(DiscoveredSource {
                name: "ShiftDelete".to_string(),
                domain: "shiftdelete.net".to_string(),
                suggested_feed: Some("https://shiftdelete.net/feed".to_string()),
                category: "Teknoloji".to_string(),
            });
            results.push(DiscoveredSource {
                name: "DonanimHaber".to_string(),
                domain: "donanimhaber.com".to_string(),
                suggested_feed: Some("https://www.donanimhaber.com/rss/tum-haberler".to_string()),
                category: "Teknoloji".to_string(),
            });
            results.push(DiscoveredSource {
                name: "LOG".to_string(),
                domain: "log.com.tr".to_string(),
                suggested_feed: Some("https://www.log.com.tr/feed/".to_string()),
                category: "Teknoloji".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Teknoblog".to_string(),
                domain: "teknoblog.com".to_string(),
                suggested_feed: Some("https://www.teknoblog.com/feed/".to_string()),
                category: "Teknoloji".to_string(),
            });
            results.push(DiscoveredSource {
                name: "CHIP Turkiye".to_string(),
                domain: "chip.com.tr".to_string(),
                suggested_feed: Some("https://www.chip.com.tr/rss".to_string()),
                category: "Teknoloji".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Swipeline".to_string(),
                domain: "swipeline.co".to_string(),
                suggested_feed: Some("https://swipeline.co/feed/".to_string()),
                category: "Girisimcilik".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Donanim Arsivi".to_string(),
                domain: "donanimarsivi.com".to_string(),
                suggested_feed: Some("https://donanimarsivi.com/feed/".to_string()),
                category: "Donanim".to_string(),
            });
            results.push(DiscoveredSource {
                name: "BTK Haber".to_string(),
                domain: "btk.gov.tr".to_string(),
                suggested_feed: Some("https://www.btk.gov.tr/rss/haberler.xml".to_string()),
                category: "Bilisim".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Pazarlamasyon".to_string(),
                domain: "pazarlamasyon.com".to_string(),
                suggested_feed: Some("https://pazarlamasyon.com/feed/".to_string()),
                category: "Dijital".to_string(),
            });
        } else if is_linux {
            results.push(DiscoveredSource {
                name: "Phoronix".to_string(),
                domain: "phoronix.com".to_string(),
                suggested_feed: Some("https://www.phoronix.com/rss.php".to_string()),
                category: "Linux".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Arch Linux News".to_string(),
                domain: "archlinux.org".to_string(),
                suggested_feed: Some("https://archlinux.org/feeds/news/".to_string()),
                category: "Linux".to_string(),
            });
            results.push(DiscoveredSource {
                name: "It's FOSS".to_string(),
                domain: "itsfoss.com".to_string(),
                suggested_feed: Some("https://itsfoss.com/feed/".to_string()),
                category: "Acik Kaynak".to_string(),
            });
            results.push(DiscoveredSource {
                name: "OMG! Ubuntu".to_string(),
                domain: "omgubuntu.co.uk".to_string(),
                suggested_feed: Some("https://www.omgubuntu.co.uk/feed".to_string()),
                category: "Linux".to_string(),
            });
            results.push(DiscoveredSource {
                name: "LWN.net".to_string(),
                domain: "lwn.net".to_string(),
                suggested_feed: Some("https://lwn.net/headlines/rss".to_string()),
                category: "Cekirdek".to_string(),
            });
        } else if is_science {
            results.push(DiscoveredSource {
                name: "Evrim Agaci".to_string(),
                domain: "evrimagaci.org".to_string(),
                suggested_feed: Some("https://evrimagaci.org/rss.xml".to_string()),
                category: "Bilim".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Phys.org".to_string(),
                domain: "phys.org".to_string(),
                suggested_feed: Some("https://phys.org/rss-feed/".to_string()),
                category: "Fizik & Bilim".to_string(),
            });
            results.push(DiscoveredSource {
                name: "ScienceDaily".to_string(),
                domain: "sciencedaily.com".to_string(),
                suggested_feed: Some("https://www.sciencedaily.com/rss/all.xml".to_string()),
                category: "Bilim".to_string(),
            });
        } else if is_finance {
            results.push(DiscoveredSource {
                name: "BloombergHT".to_string(),
                domain: "bloomberght.com".to_string(),
                suggested_feed: Some("https://www.bloomberght.com/rss".to_string()),
                category: "Ekonomi".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Dunya Gazetesi".to_string(),
                domain: "dunya.com".to_string(),
                suggested_feed: Some("https://www.dunya.com/rss".to_string()),
                category: "Finans".to_string(),
            });
        } else {
            // Default global tech & programming mix
            results.push(DiscoveredSource {
                name: "Hacker News".to_string(),
                domain: "news.ycombinator.com".to_string(),
                suggested_feed: Some("https://news.ycombinator.com/rss".to_string()),
                category: "Yazilim & Girisim".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Ars Technica".to_string(),
                domain: "arstechnica.com".to_string(),
                suggested_feed: Some("https://feeds.arstechnica.com/arstechnica/index".to_string()),
                category: "Teknoloji".to_string(),
            });
            results.push(DiscoveredSource {
                name: "The Verge".to_string(),
                domain: "theverge.com".to_string(),
                suggested_feed: Some("https://www.theverge.com/rss/index.xml".to_string()),
                category: "Teknoloji".to_string(),
            });
        }

        results
    }

    async fn query_gemini_for_sources(prompt: &str, api_key: &str) -> Result<Vec<DiscoveredSource>, String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            api_key
        );

        let system_instruction = "Sen bir haber kaynagi kesif motorusun. Kullanicinin dogal dil istegini analiz et ve en yuksek kaliteli haber sitelerinin adini, alan adini, RSS akisini (varsa) ve kategorisini dondur. Sadece gecerli bir JSON dizisi dondur: [{\"name\":\"...\",\"domain\":\"...\",\"suggested_feed\":\"...\",\"category\":\"...\"}]";

        let payload = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": format!("{}\n\nKullanici Istegi: {}", system_instruction, prompt)
                }]
            }],
            "generationConfig": {
                "response_mime_type": "application/json"
            }
        });

        let client = reqwest::Client::new();
        let res = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json_body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        if let Some(text) = json_body["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            if let Ok(parsed) = serde_json::from_str::<Vec<DiscoveredSource>>(text) {
                return Ok(parsed);
            }
        }

        Err("Gemini yaniti ayrilamadi".to_string())
    }

    async fn query_gemini_summary(
        title: &str,
        content: &str,
        api_key: &str,
    ) -> Result<AiSummaryResult, String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            api_key
        );

        let prompt = format!(
            "Haber Basligi: {}\n\nIcerik:\n{}\n\nGorev: Bu haberi analiz et. Varsa clickbait/sansasyonel basligi tamamen tarafsiz ve gercekci bir basliga donustur ('neutral_title'). Haberin en can alici noktalarini tam 3 maddelik hap cumleler halinde 'key_points' dizisine yaz. Yaniti yalnizca su JSON semasiyla dondur:\n{{\"neutral_title\":\"...\",\"key_points\":[\"...\",\"...\",\"...\"]}}",
            title,
            if content.len() > 6000 { &content[..6000] } else { content }
        );

        let payload = serde_json::json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }],
            "generationConfig": {
                "response_mime_type": "application/json"
            }
        });

        let client = reqwest::Client::new();
        let res = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json_body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        if let Some(text) = json_body["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            #[derive(Deserialize)]
            struct Partial {
                neutral_title: String,
                key_points: Vec<String>,
            }
            if let Ok(parsed) = serde_json::from_str::<Partial>(text) {
                let word_count = content.split_whitespace().count();
                let reading_time = if word_count == 0 { 1 } else { (word_count + 199) / 200 };
                return Ok(AiSummaryResult {
                    neutral_title: parsed.neutral_title,
                    key_points: parsed.key_points,
                    reading_time_mins: reading_time,
                });
            }
        }

        Err("Gemini ozet yaniti ayrilamadi".to_string())
    }

    async fn query_ollama_for_sources(prompt: &str) -> Result<Vec<DiscoveredSource>, String> {
        let url = "http://127.0.0.1:11434/api/generate";
        let body = serde_json::json!({
            "model": "llama3",
            "prompt": format!("As a news source engine, return ONLY a JSON array of sources for: '{}'. Format: [{{\"name\":\"...\",\"domain\":\"...\",\"suggested_feed\":\"...\",\"category\":\"...\"}}]", prompt),
            "stream": false,
            "format": "json"
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .map_err(|e| e.to_string())?;

        let res = client.post(url).json(&body).send().await.map_err(|e| e.to_string())?;
        let json_body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        if let Some(resp_str) = json_body["response"].as_str() {
            if let Ok(parsed) = serde_json::from_str::<Vec<DiscoveredSource>>(resp_str) {
                return Ok(parsed);
            }
        }
        Err("Ollama erisilemedi".to_string())
    }

    async fn query_ollama_summary(title: &str, content: &str) -> Result<AiSummaryResult, String> {
        let url = "http://127.0.0.1:11434/api/generate";
        let body = serde_json::json!({
            "model": "llama3",
            "prompt": format!("Analyze this news: '{}'. Text: '{}'. Return JSON: {{\"neutral_title\":\"...\", \"key_points\":[\"...\", \"...\", \"...\"]}}", title, if content.len() > 3000 { &content[..3000] } else { content }),
            "stream": false,
            "format": "json"
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(4))
            .build()
            .map_err(|e| e.to_string())?;

        let res = client.post(url).json(&body).send().await.map_err(|e| e.to_string())?;
        let json_body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        if let Some(resp_str) = json_body["response"].as_str() {
            #[derive(Deserialize)]
            struct Partial {
                neutral_title: String,
                key_points: Vec<String>,
            }
            if let Ok(parsed) = serde_json::from_str::<Partial>(resp_str) {
                let word_count = content.split_whitespace().count();
                let reading_time = if word_count == 0 { 1 } else { (word_count + 199) / 200 };
                return Ok(AiSummaryResult {
                    neutral_title: parsed.neutral_title,
                    key_points: parsed.key_points,
                    reading_time_mins: reading_time,
                });
            }
        }
        Err("Ollama ozet basarisiz".to_string())
    }
}
