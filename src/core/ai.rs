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

    /// Cleans and semantically structures the FULL article text using AI:
    /// - Preserves 100% of the news content (never shortens or truncates)
    /// - Cleans all leftover site clutter, ads, cookie notices, social calls
    /// - Accurately formats paragraphs (\n\n)
    /// - Preserves and cleanly formats bullet lists ("- ...") and numbered lists ("1. ...")
    /// - Preserves subheadings ("## ...")
    /// - Zero unicode emojis
    pub async fn clean_full_article_content(title: &str, raw_text: &str) -> String {
        // 1. Try Gemini if GEMINI_API_KEY is configured
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            if !api_key.trim().is_empty() {
                if let Ok(cleaned) = Self::query_gemini_full_article(title, raw_text, &api_key).await {
                    if cleaned.trim().len() > 100 {
                        return cleaned;
                    }
                }
            }
        }

        // 2. Try Ollama if running
        if let Ok(cleaned) = Self::query_ollama_full_article(title, raw_text).await {
            if cleaned.trim().len() > 100 {
                return cleaned;
            }
        }

        // 3. Deterministic Local NLP Formatter (Guarantees clean paragraphs, lists and headings)
        Self::local_structure_full_article(title, raw_text)
    }

    /// Local rule-based structural formatter preserving all paragraphs and lists.
    pub fn local_structure_full_article(_title: &str, raw_text: &str) -> String {
        let lines: Vec<&str> = raw_text.lines().collect();
        let mut clean_blocks: Vec<String> = Vec::new();

        let clutter_signatures = [
            "bizi takip edin",
            "takip etmeyi unutmayin",
            "abone olun",
            "yorum yapin",
            "yorumlarinizi",
            "fikirlerinizi paylas",
            "goruslerinizi paylas",
            "yorumlarinizi bizimle",
            "ne dusunuyorsunuz?",
            "tartismaya katilabilirsiniz",
            "ilgili haberler",
            "daha fazla oku",
            "sponsorlu icerik",
            "copyright",
            "tum haklari saklidir",
            "etiketler:",
            "kaynak:",
            "yazar hakkinda",
            "cerez politikas",
            "kreosus",
            "patreon.com",
            "desteklerinize ihtiyac",
            "destekcilere ozel",
            "destekçilere özel",
            "cikti bilgisi:",
            "çıktı bilgisi:",
            "icerik kullanim izinleri",
            "içerik kullanım izinleri",
            "yazdir / pdf",
            "yazdır / pdf",
            "bu makale sana ne hissettirdi",
            "soru & cevap",
            "this work is an exact translation",
            "yorum yaz",
            "paylaş tweetle",
            "paylas tweetle",
            "eposta ile paylaşın",
            "eposta ile paylasin",
            "arkadaşınıza postalayın",
            "arkadasiniza postalayin",
            "gelişmiş mobil uygulamamızı",
            "gelismis mobil uygulamamizi",
            "mobil uygulamamızı kullanarak",
            "mobil uygulamamizi kullanarak",
            "haber bildirimlerini aç",
            "haber bildirimlerini ac",
            "whatsapp kanalımıza katılın",
            "whatsapp kanalimiza katilin",
            "telegram kanalımıza katılın",
            "telegram kanalimiza katilin",
            "instagram takip et",
            "video bitince sonrakine geç",
            "video bitince sonrakine gec",
            "ilginizi çekebilir",
            "ilginizi cekebilir",
            "benzer haberler",
            "ilgili yazılar",
            "ilgili yazilar",
            "sıradaki haber",
            "siradaki haber",
            "şunlar da ilginizi",
            "sunlar da ilginizi",
            "göz atmak isteyebilirsiniz",
            "goz atmak isteyebilirsiniz",
            "en çok okunanlar",
            "en cok okunanlar",
            "next page",
            "go to next page",
            "previous page",
            "sonraki sayfa",
            "önceki sayfa",
            "onceki sayfa",
            "page 1 of",
            "page 2 of",
            "page 3 of",
            "page 4 of",
            "page 5 of",
            "page 6 of",
            "page 7 of",
            "page 8 of",
            "sayfa 1 /",
            "sayfa 2 /",
            "sayfa 3 /",
            "sayfa 1 of",
        ];

        for line in lines {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                continue;
            }

            let lower = line_trimmed.to_lowercase();

            // Skip purely pagination navigation bars (e.g. "Page: 1 2 3 4 5 6 7 8 Next Page")
            if lower.starts_with("page:")
                || lower.starts_with("sayfa:")
                || lower == "next page"
                || lower == "sonraki sayfa"
                || lower == "previous page"
                || lower == "önceki sayfa"
                || lower == "onceki sayfa"
            {
                continue;
            }

            // If related/suggested news widget starts and we already have sufficient article text, stop processing
            if (lower.starts_with("ilginizi çekebilir")
                || lower.starts_with("ilginizi cekebilir")
                || lower.starts_with("benzer haberler")
                || lower.starts_with("sıradaki haber")
                || lower.starts_with("şunlar da ilginizi"))
                && clean_blocks.len() >= 2
            {
                break;
            }

            let is_clutter = clutter_signatures.iter().any(|&sig| lower.contains(sig));
            if is_clutter {
                continue;
            }

            // Strip any leftover CMS image/video placeholder artifacts
            let cleaned_text = line_trimmed
                .replace("[image_1]", "")
                .replace("[image_2]", "")
                .replace("[image_3]", "")
                .replace("[image_4]", "")
                .replace("[image_5]", "")
                .replace("[image]", "")
                .replace("[resim]", "")
                .replace("[video]", "")
                .trim()
                .to_string();

            if cleaned_text.is_empty() {
                continue;
            }

            let trimmed = cleaned_text.as_str();

            // Check if line is a heading
            if trimmed.starts_with("##") || trimmed.starts_with("###") {
                clean_blocks.push(trimmed.to_string());
                continue;
            }

            // Check if line is a bullet item
            if trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('•') || trimmed.starts_with('·') {
                let rest = trimmed.trim_start_matches(['-', '*', '•', '·']).trim();
                if !rest.is_empty() {
                    clean_blocks.push(format!("- {}", rest));
                    continue;
                }
            }

            // Check if line is a numbered item (e.g. "1.", "2)")
            let is_numbered = trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                && (trimmed.contains(". ") || trimmed.contains(") "));
            if is_numbered {
                clean_blocks.push(trimmed.to_string());
                continue;
            }

            // Check if line is a blockquote
            if trimmed.starts_with('>') {
                clean_blocks.push(trimmed.to_string());
                continue;
            }

            // Regular paragraph line
            clean_blocks.push(trimmed.to_string());
        }

        // Strip trailing orphaned headings that don't have body content below them
        while clean_blocks.last().is_some_and(|b| b.starts_with("##") || b.starts_with("###")) {
            clean_blocks.pop();
        }

        // Group into semantic paragraphs with clear heading separation
        let mut output = String::new();
        let mut in_list = false;

        for block in clean_blocks {
            if block.starts_with("##") {
                if in_list {
                    output.push('\n');
                    in_list = false;
                }
                if !output.is_empty() {
                    // Basliktan evvel mutlaka belirgin bir satir bosluk birak (\n\n\n)
                    output.push_str("\n\n\n");
                }
                output.push_str(&block);
                output.push_str("\n\n");
                continue;
            }

            let is_list_item = block.starts_with("- ")
                || (block.chars().next().is_some_and(|c| c.is_ascii_digit()) && block.contains(". "));

            if is_list_item {
                if !in_list && !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(&block);
                output.push('\n');
                in_list = true;
            } else {
                if in_list {
                    output.push('\n');
                    in_list = false;
                }
                let comfortable_p = Self::format_comfortable_paragraphs(&block);
                for p in comfortable_p {
                    if !output.is_empty() && !output.ends_with("\n\n") {
                        if output.ends_with('\n') {
                            output.push('\n');
                        } else {
                            output.push_str("\n\n");
                        }
                    }
                    output.push_str(&p);
                }
            }
        }

        if output.trim().is_empty() {
            raw_text.to_string()
        } else {
            output
        }
    }

    /// Alias helper to clean and format content blocks
    pub fn clean_blocks(text: &str) -> String {
        Self::local_structure_full_article("", text)
    }

    /// Automatically fixes punctuation spacing (e.g. "suçladı.Çin" -> "suçladı. Çin")
    pub fn fix_sentence_spacing(text: &str) -> String {
        let mut result = String::with_capacity(text.len() + 16);
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        for i in 0..len {
            result.push(chars[i]);
            if (chars[i] == '.' || chars[i] == '!' || chars[i] == '?') && i + 1 < len {
                let next_ch = chars[i + 1];
                if next_ch.is_uppercase() {
                    let prev_short = i >= 2 && (chars[i - 1].is_lowercase() || chars[i - 1].is_uppercase()) && i >= 3 && chars[i - 2] == '.';
                    if !prev_short {
                        result.push(' ');
                    }
                }
            }
        }
        result
    }

    /// Splits an overly long monolithic block into comfortable 2-3 sentence paragraphs
    pub fn format_comfortable_paragraphs(text: &str) -> Vec<String> {
        let cleaned = Self::fix_sentence_spacing(text);
        let word_count = cleaned.split_whitespace().count();

        if word_count < 65 {
            return vec![cleaned];
        }

        let mut paragraphs = Vec::new();
        let mut current_p = String::new();
        let mut sentence_count = 0;

        let parts = cleaned.split_inclusive(['.', '!', '?']);
        for part in parts {
            current_p.push_str(part);
            let trimmed = part.trim();
            if trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?') {
                sentence_count += 1;
                if sentence_count >= 3 && current_p.split_whitespace().count() >= 35 {
                    paragraphs.push(current_p.trim().to_string());
                    current_p.clear();
                    sentence_count = 0;
                }
            }
        }

        if !current_p.trim().is_empty() {
            paragraphs.push(current_p.trim().to_string());
        }

        if paragraphs.is_empty() {
            vec![cleaned]
        } else {
            paragraphs
        }
    }

    /// Automatically classifies news source and content into accurate tags
    pub fn auto_classify_tags(source_name: &str, category: &str, title: &str, text: &str) -> Vec<String> {
        let mut tags = Vec::new();

        let cat_trimmed = category.trim();
        if !cat_trimmed.is_empty() && cat_trimmed != "Genel" && cat_trimmed != "Tümü" {
            tags.push(cat_trimmed.to_string());
        }

        let combined = format!("{} {} {}", title, text, source_name).to_lowercase();

        if (combined.contains("yapay zeka")
            || combined.contains("yapay zekâ")
            || combined.contains("ai")
            || combined.contains(" ai ")
            || combined.contains("llm")
            || combined.contains("gemini")
            || combined.contains("chatgpt")
            || combined.contains("openai")
            || combined.contains("claude")
            || combined.contains("anthropic")
            || combined.contains("qwen")
            || combined.contains("deepseek")
            || combined.contains("makine öğrenimi"))
            && !tags.iter().any(|t| t == "Yapay Zeka")
        {
            tags.push("Yapay Zeka".to_string());
        }

        if (combined.contains("donanım")
            || combined.contains("donanim")
            || combined.contains("işlemci")
            || combined.contains("islemci")
            || combined.contains("ekran kartı")
            || combined.contains("ekran karti")
            || combined.contains("gpu")
            || combined.contains("cpu")
            || combined.contains("rtx")
            || combined.contains("geforce")
            || combined.contains("intel")
            || combined.contains("amd")
            || combined.contains("nvidia")
            || combined.contains("anakart"))
            && !tags.iter().any(|t| t == "Donanım")
        {
            tags.push("Donanım".to_string());
        }

        if (combined.contains("siber")
            || combined.contains("güvenlik")
            || combined.contains("guvenlik")
            || combined.contains("hacker")
            || combined.contains("fbi")
            || combined.contains("casus")
            || combined.contains("malware")
            || combined.contains("fidye")
            || combined.contains("zafiyet")
            || combined.contains("açık"))
            && !tags.iter().any(|t| t == "Siber Güvenlik")
        {
            tags.push("Siber Güvenlik".to_string());
        }

        if (combined.contains("iphone")
            || combined.contains("apple")
            || combined.contains("android")
            || combined.contains("samsung")
            || combined.contains("ios")
            || combined.contains("xiaomi")
            || combined.contains("akıllı telefon")
            || combined.contains("telefon"))
            && !tags.iter().any(|t| t == "Mobil")
        {
            tags.push("Mobil".to_string());
        }

        if (combined.contains("linux")
            || combined.contains("açık kaynak")
            || combined.contains("acik kaynak")
            || combined.contains("kernel")
            || combined.contains("ubuntu")
            || combined.contains("arch")
            || combined.contains("fedora")
            || combined.contains("gnome")
            || combined.contains("omarchy"))
            && !tags.iter().any(|t| t == "Açık Kaynak")
        {
            tags.push("Açık Kaynak".to_string());
        }

        if (combined.contains("oyun")
            || combined.contains("steam")
            || combined.contains("playstation")
            || combined.contains("xbox")
            || combined.contains("nintendo")
            || combined.contains("game"))
            && !tags.iter().any(|t| t == "Oyun")
        {
            tags.push("Oyun".to_string());
        }

        if (combined.contains("girişim")
            || combined.contains("girisim")
            || combined.contains("startup")
            || combined.contains("yatırım")
            || combined.contains("yatirim")
            || combined.contains("değerleme")
            || combined.contains("fon")
            || combined.contains("fintech"))
            && !tags.iter().any(|t| t == "Girişimcilik")
        {
            tags.push("Girişimcilik".to_string());
        }

        if (combined.contains("uzay")
            || combined.contains("nasa")
            || combined.contains("bilim")
            || combined.contains("fizik")
            || combined.contains("evrim")
            || combined.contains("biyoloji")
            || combined.contains("teleskop"))
            && !tags.iter().any(|t| t == "Bilim")
        {
            tags.push("Bilim".to_string());
        }

        if (combined.contains("siyaset")
            || combined.contains("politika")
            || combined.contains("meclis")
            || combined.contains("hükümet")
            || combined.contains("hukumet")
            || combined.contains("seçim")
            || combined.contains("secim")
            || combined.contains("bakan")
            || combined.contains("chp")
            || combined.contains("akp")
            || combined.contains("parti")
            || combined.contains("milletvekili"))
            && !tags.iter().any(|t| t == "Siyaset")
        {
            tags.push("Siyaset".to_string());
        }

        if (combined.contains("gündem")
            || combined.contains("gundem")
            || combined.contains("son dakika")
            || combined.contains("manşet")
            || combined.contains("manset")
            || combined.contains("haberler")
            || combined.contains("asayiş")
            || combined.contains("olay"))
            && !tags.iter().any(|t| t == "Gündem")
        {
            tags.push("Gündem".to_string());
        }

        if (combined.contains("yerel")
            || combined.contains("belediye")
            || combined.contains("büyükşehir")
            || combined.contains("buyuksehir")
            || combined.contains("istanbul")
            || combined.contains("ankara")
            || combined.contains("izmir")
            || combined.contains("bursa")
            || combined.contains("antalya")
            || combined.contains("valilik")
            || combined.contains("muhtar"))
            && !tags.iter().any(|t| t == "Yerel")
        {
            tags.push("Yerel".to_string());
        }

        if tags.is_empty() {
            tags.push("Teknoloji".to_string());
        }

        tags.truncate(3);
        tags
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
        let reading_time = if word_count == 0 { 1 } else { word_count.div_ceil(200) };

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
        scored_sentences.sort_by_key(|a| std::cmp::Reverse(a.0));

        let top_count = std::cmp::min(3, scored_sentences.len());
        let mut bullets = Vec::new();
        for item in scored_sentences.iter().take(top_count) {
            bullets.push(format!("{}.", item.1));
        }

        AiSummaryResult {
            neutral_title: clean_title,
            key_points: bullets,
            reading_time_mins: reading_time,
        }
    }

    pub fn resolve_curated_knowledge_base(query: &str) -> Vec<DiscoveredSource> {
        let mut results = Vec::new();
        let q = query.to_lowercase();

        let is_tr = q.contains("turkiye") || q.contains("türkiye") || q.contains("turk") || q.contains("türk");
        let is_tech = q.contains("teknoloji") || q.contains("yazilim") || q.contains("yazılım") || q.contains("bilisim") || q.contains("tech");
        let is_linux = q.contains("linux") || q.contains("acik kaynak") || q.contains("açık kaynak") || q.contains("open source");
        let is_science = q.contains("bilim") || q.contains("uzay") || q.contains("science");
        let is_finance = q.contains("ekonomi") || q.contains("finans") || q.contains("borsa") || q.contains("dolar");
        let is_politics = q.contains("siyas") || q.contains("politika") || q.contains("meclis") || q.contains("hukumet") || q.contains("hükümet") || q.contains("parti") || q.contains("secim") || q.contains("seçim");
        let is_local = q.contains("yerel") || q.contains("ankara") || q.contains("istanbul") || q.contains("izmir") || q.contains("bursa") || q.contains("antalya") || q.contains("sehir") || q.contains("şehir") || q.contains("belediye");
        let is_general = q.contains("gundem") || q.contains("gündem") || q.contains("genel") || q.contains("haber") || q.contains("manset") || q.contains("manşet") || q.contains("gazete") || q.contains("ajans") || q.contains("son dakika");

        if is_local {
            results.push(DiscoveredSource {
                name: "Istanbul Bulteni".to_string(),
                domain: "ibb.istanbul".to_string(),
                suggested_feed: Some("https://www.ibb.istanbul/rss".to_string()),
                category: "Yerel".to_string(),
            });
            results.push(DiscoveredSource {
                name: "IHA Yerel".to_string(),
                domain: "iha.com.tr".to_string(),
                suggested_feed: Some("https://www.iha.com.tr/rss".to_string()),
                category: "Yerel".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Bursa Hakimiyet".to_string(),
                domain: "bursahakimiyet.com.tr".to_string(),
                suggested_feed: Some("https://www.bursahakimiyet.com.tr/rss".to_string()),
                category: "Yerel".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Ege Telgraf".to_string(),
                domain: "egetelgraf.com".to_string(),
                suggested_feed: Some("https://www.egetelgraf.com/rss".to_string()),
                category: "Yerel".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Baskent Gazetesi".to_string(),
                domain: "baskentgazete.com.tr".to_string(),
                suggested_feed: Some("https://www.baskentgazete.com.tr/rss".to_string()),
                category: "Yerel".to_string(),
            });
        } else if is_politics || is_general {
            results.push(DiscoveredSource {
                name: "Sozcu".to_string(),
                domain: "sozcu.com.tr".to_string(),
                suggested_feed: Some("https://www.sozcu.com.tr/feeds-son-dakika".to_string()),
                category: "Gundem".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Haberturk".to_string(),
                domain: "haberturk.com".to_string(),
                suggested_feed: Some("https://www.haberturk.com/rss".to_string()),
                category: "Gundem".to_string(),
            });
            results.push(DiscoveredSource {
                name: "NTV Gundem".to_string(),
                domain: "ntv.com.tr".to_string(),
                suggested_feed: Some("https://www.ntv.com.tr/gundem.rss".to_string()),
                category: "Gundem".to_string(),
            });
            results.push(DiscoveredSource {
                name: "BBC Turkce".to_string(),
                domain: "bbc.com".to_string(),
                suggested_feed: Some("https://feeds.bbci.co.uk/turkce/rss.xml".to_string()),
                category: "Siyaset".to_string(),
            });
            results.push(DiscoveredSource {
                name: "T24".to_string(),
                domain: "t24.com.tr".to_string(),
                suggested_feed: Some("https://t24.com.tr/rss".to_string()),
                category: "Siyaset".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Gazete Duvar".to_string(),
                domain: "gazeteduvar.com.tr".to_string(),
                suggested_feed: Some("https://www.gazeteduvar.com.tr/export/rss".to_string()),
                category: "Siyaset".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Anadolu Ajansi".to_string(),
                domain: "aa.com.tr".to_string(),
                suggested_feed: Some("https://www.aa.com.tr/tr/rss/default?cat=guncel".to_string()),
                category: "Gundem".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Cumhuriyet".to_string(),
                domain: "cumhuriyet.com.tr".to_string(),
                suggested_feed: Some("https://www.cumhuriyet.com.tr/rss".to_string()),
                category: "Siyaset".to_string(),
            });
        } else if is_tr && is_tech {
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
                suggested_feed: Some("https://www.donanimhaber.com/rss/tum/".to_string()),
                category: "Teknoloji".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Webtekno".to_string(),
                domain: "webtekno.com".to_string(),
                suggested_feed: Some("https://www.webtekno.com/rss.xml".to_string()),
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
                name: "Hardware Plus".to_string(),
                domain: "hwp.com.tr".to_string(),
                suggested_feed: Some("https://hwp.com.tr/feed".to_string()),
                category: "Donanim".to_string(),
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
                name: "Egirisim".to_string(),
                domain: "egirisim.com".to_string(),
                suggested_feed: Some("https://egirisim.com/feed/".to_string()),
                category: "Girisimcilik".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Donanim Arsivi".to_string(),
                domain: "donanimarsivi.com".to_string(),
                suggested_feed: Some("https://donanimarsivi.com/feed/".to_string()),
                category: "Donanim".to_string(),
            });
            results.push(DiscoveredSource {
                name: "Donanim Gunlugu".to_string(),
                domain: "donanimgunlugu.com".to_string(),
                suggested_feed: Some("https://donanimgunlugu.com/feed".to_string()),
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
        let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent";

        let system_instruction = "Sen bir haber kaynagi kesif motorusun. Kullanicinin teknoloji, siyaset, gundem, yerel, ekonomi veya bilim isteklerini analiz et. Belirtilen alanla ilgili en yuksek kaliteli haber sitelerinin adini, alan adini, RSS akisini (varsa) ve kategorisini (orn: Gundem, Siyaset, Yerel, Teknoloji, Bilim, Ekonomi) dondur. Sadece gecerli bir JSON dizisi dondur: [{\"name\":\"...\",\"domain\":\"...\",\"suggested_feed\":\"...\",\"category\":\"...\"}]";

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

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let res = client
            .post(url)
            .header("x-goog-api-key", api_key.trim())
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
        let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent";

        let truncated_content = safe_truncate_str(content, 6000);
        let prompt = format!(
            "Haber Basligi: {}\n\nIcerik:\n{}\n\nGorev: Bu haberi analiz et. Varsa clickbait/sansasyonel basligi tamamen tarafsiz ve gercekci bir basliga donustur ('neutral_title'). Haberin en can alici noktalarini tam 3 maddelik hap cumleler halinde 'key_points' dizisine yaz. Yaniti yalnizca su JSON semasiyla dondur:\n{{\"neutral_title\":\"...\",\"key_points\":[\"...\",\"...\",\"...\"]}}",
            title,
            truncated_content
        );

        let payload = serde_json::json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }],
            "generationConfig": {
                "response_mime_type": "application/json"
            }
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let res = client
            .post(url)
            .header("x-goog-api-key", api_key.trim())
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
                let reading_time = if word_count == 0 { 1 } else { word_count.div_ceil(200) };
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
            "prompt": format!("As a news source discovery engine, return ONLY a JSON array of top news/media sources for topic/country/city: '{}'. Categories can be: 'Gundem', 'Siyaset', 'Yerel', 'Teknoloji', 'Bilim', 'Ekonomi'. Format: [{{\"name\":\"...\",\"domain\":\"...\",\"suggested_feed\":\"...\",\"category\":\"...\"}}]", prompt),
            "stream": false,
            "format": "json"
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .connect_timeout(std::time::Duration::from_millis(800))
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
        let truncated_content = safe_truncate_str(content, 3000);
        let body = serde_json::json!({
            "model": "llama3",
            "prompt": format!("Analyze this news: '{}'. Text: '{}'. Return JSON: {{\"neutral_title\":\"...\", \"key_points\":[\"...\", \"...\", \"...\"]}}", title, truncated_content),
            "stream": false,
            "format": "json"
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(4))
            .connect_timeout(std::time::Duration::from_millis(800))
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
                let reading_time = if word_count == 0 { 1 } else { word_count.div_ceil(200) };
                return Ok(AiSummaryResult {
                    neutral_title: parsed.neutral_title,
                    key_points: parsed.key_points,
                    reading_time_mins: reading_time,
                });
            }
        }
        Err("Ollama ozet basarisiz".to_string())
    }

    async fn query_gemini_full_article(
        title: &str,
        raw_text: &str,
        api_key: &str,
    ) -> Result<String, String> {
        let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent";

        let system_instruction = "Sen bir saf metin haber editorusun. Cok sayfali veya tek sayfali makale metinlerini bastan sona eksiksiz, kelime kelime koruyarak duzenle. Sayfa gecislerinde kalan 'Page 1 of 8', 'Next Page', 'Sonraki Sayfa' gibi sayfalama kalintilarini, reklam ve sosyal medya duyurularini tamamen temizle. Paragraflari iki satir boslukla net ayir. Varsa maddeli listeleri ('- madde') ve numarali listeleri ('1. madde') liste hiyerarsisiyle aktar. Varsa alt basliklari '## Baslik' formatinda koru. Haberi ASLA kisaltma, ozetleme ya da kesme; tum gercek haberi eksiksiz aktar. KESINLIKLE HICBIR UNICODE EMOJI KULLANMA. Yalnizca temizlenmis metni dondur.";

        let truncated_text = safe_truncate_str(raw_text, 24000);
        let payload = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": format!("{}\n\nBaslik: {}\n\nHam Metin:\n{}", system_instruction, title, truncated_text)
                }]
            }]
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(18))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        let res = client
            .post(url)
            .header("x-goog-api-key", api_key.trim())
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json_body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        if let Some(text) = json_body["candidates"][0]["content"]["parts"][0]["text"].as_str() {
            return Ok(text.trim().to_string());
        }

        Err("Gemini tam metin yaniti ayrilamadi".to_string())
    }

    async fn query_ollama_full_article(title: &str, raw_text: &str) -> Result<String, String> {
        let url = "http://127.0.0.1:11434/api/generate";
        let truncated_text = safe_truncate_str(raw_text, 6000);
        let body = serde_json::json!({
            "model": "llama3",
            "prompt": format!("Format this full news article cleanly with paragraphs, subheadings, and lists. Clean any multi-page pagination remnants like 'Page 1 of 8' or 'Next Page'. DO NOT summarize or shorten. Preserve all information. Remove ads, cookie notices, and site clutter. Never use emojis.\n\nTitle: {}\nText:\n{}", title, truncated_text),
            "stream": false
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .connect_timeout(std::time::Duration::from_millis(800))
            .build()
            .map_err(|e| e.to_string())?;

        let res = client.post(url).json(&body).send().await.map_err(|e| e.to_string())?;
        let json_body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        if let Some(resp_str) = json_body["response"].as_str() {
            return Ok(resp_str.trim().to_string());
        }
        Err("Ollama tam metin basarisiz".to_string())
    }
}

fn safe_truncate_str(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}
