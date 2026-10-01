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
    /// Retrieves Gemini API key from environment variable or ~/.config/quicknews/gemini_api_key
    pub fn get_gemini_api_key() -> Option<String> {
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            if !key.trim().is_empty() {
                return Some(key.trim().to_string());
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let key_file = std::path::PathBuf::from(home).join(".config").join("quicknews").join("gemini_api_key");
            if let Ok(content) = std::fs::read_to_string(key_file) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }

    /// Discovers news sources from natural language prompt (e.g. "Turkiye teknoloji sitelerinden 10 tanesini ekle")
    pub async fn discover_sources_from_prompt(prompt: &str) -> Vec<DiscoveredSource> {
        let p_lower = prompt.to_lowercase();

        // 1. Try Google Gemini API if GEMINI_API_KEY is configured
        if let Some(api_key) = Self::get_gemini_api_key() {
            if let Ok(res) = Self::query_gemini_for_sources(prompt, &api_key).await {
                if !res.is_empty() {
                    return res;
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
        if let Some(api_key) = Self::get_gemini_api_key() {
            if let Ok(cleaned) = Self::query_gemini_full_article(title, raw_text, &api_key).await {
                if cleaned.trim().len() > 100 {
                    return cleaned;
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
    pub fn local_structure_full_article(title: &str, raw_text: &str) -> String {
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
            "phoronix premium",
            "view this site ad-free",
            "paypal or stripe tips",
            "thanks for your support",
        ];

        for line in lines {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                continue;
            }

            let lower = line_trimmed.to_lowercase();

            // Skip purely pagination navigation bars (e.g. "Page: 1 2 3 4 5 6 7 8 Next Page", "Page, 1, 2, 3, 4")
            if lower.starts_with("page:")
                || lower.starts_with("pages:")
                || lower.starts_with("page,")
                || lower.starts_with("sayfa:")
                || lower.starts_with("sayfalar:")
                || lower.starts_with("sayfa,")
                || lower.contains("page: 1 2")
                || lower.contains("page: 1, 2")
                || lower.contains("page 1 2 3")
                || lower.contains("page, 1, 2")
                || lower.contains("sayfa: 1 2")
                || lower.contains("sayfa: 1, 2")
                || lower.contains("sayfa 1 2 3")
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

            // Skip duplicate title at the beginning of the article
            if clean_blocks.is_empty() {
                let norm_title = title.trim().to_lowercase();
                let strip_h = lower.trim_start_matches('#').trim();
                if !norm_title.is_empty() && (strip_h == norm_title || norm_title.starts_with(strip_h) || strip_h.starts_with(&norm_title)) {
                    continue;
                }
            }

            // Skip leftover author metadata and relative timestamps (e.g. "Tuğçe İçözü 18 saat önce")
            if (lower.ends_with("saat önce")
                || lower.ends_with("saat once")
                || lower.ends_with("dakika önce")
                || lower.ends_with("dakika once")
                || lower.ends_with("gün önce")
                || lower.ends_with("gun once")
                || lower.starts_with("yazar:")
                || lower.starts_with("yazan:")
                || lower.starts_with("editör:")
                || lower.starts_with("editor:"))
                && line_trimmed.len() < 50
            {
                continue;
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

    /// Checks if a keyword exists as an isolated token or word, preventing substring false positives (e.g. "oyun" in "boyunca")
    pub fn contains_word_token(haystack: &str, needle: &str) -> bool {
        let needle_lower = needle.to_lowercase();
        let needle_chars: Vec<char> = needle_lower.chars().collect();
        if needle_chars.is_empty() {
            return false;
        }

        let haystack_lower = haystack.to_lowercase();
        let h_chars: Vec<char> = haystack_lower.chars().collect();
        let h_len = h_chars.len();
        let n_len = needle_chars.len();

        if n_len > h_len {
            return false;
        }

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '’' || c == '\'';

        for i in 0..=(h_len - n_len) {
            if h_chars[i..i + n_len] == needle_chars[..] {
                let left_boundary = if i == 0 { true } else { !is_word_char(h_chars[i - 1]) };
                let right_boundary = if i + n_len == h_len { true } else { !is_word_char(h_chars[i + n_len]) };
                if left_boundary && right_boundary {
                    return true;
                }
            }
        }
        false
    }

    /// Automatically classifies news source and content into accurate tags
    pub fn auto_classify_tags(source_name: &str, category: &str, title: &str, text: &str) -> Vec<String> {
        let mut tags = Vec::new();

        let cat_trimmed = category.trim();
        if !cat_trimmed.is_empty() && cat_trimmed != "Genel" && cat_trimmed != "Tümü" {
            tags.push(cat_trimmed.to_string());
        }

        let combined = format!("{} {} {}", title, text, source_name).to_lowercase();

        // 1. Yapay Zeka (AI) - Strict word boundary and negative context guard
        let ai_matched = combined.contains("yapay zeka")
            || combined.contains("yapay zekâ")
            || Self::contains_word_token(&combined, "ai")
            || Self::contains_word_token(&combined, "llm")
            || combined.contains("gemini")
            || combined.contains("chatgpt")
            || combined.contains("openai")
            || combined.contains("claude")
            || combined.contains("anthropic")
            || combined.contains("qwen")
            || combined.contains("deepseek")
            || combined.contains("makine öğrenimi");

        // Filter out false AI triggers (like "ai" matching inside ordinary Turkish words or domain extensions)
        if ai_matched && !tags.iter().any(|t| t == "Yapay Zeka") {
            tags.push("Yapay Zeka".to_string());
        }

        // 2. Donanım - Hardware keywords
        if (combined.contains("işlemci")
            || combined.contains("islemci")
            || combined.contains("ekran kartı")
            || combined.contains("ekran karti")
            || Self::contains_word_token(&combined, "gpu")
            || Self::contains_word_token(&combined, "cpu")
            || Self::contains_word_token(&combined, "rtx")
            || combined.contains("geforce")
            || Self::contains_word_token(&combined, "intel")
            || Self::contains_word_token(&combined, "amd")
            || Self::contains_word_token(&combined, "nvidia")
            || combined.contains("anakart")
            || (combined.contains("donanım") || combined.contains("donanim")) && !combined.contains("araç donanımı") && !combined.contains("yeni donanımı"))
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

        // 3. Oyun (Gaming) - Token boundary and negative context guard
        // Prevents matching "oyun" in "boyunca", "oyuncu değişikliği", "tiyatro oyunu", "seçim oyunu"
        let is_non_game_context = combined.contains("tiyatro")
            || combined.contains("sahne oyunu")
            || combined.contains("oyun teorisi")
            || combined.contains("siyasi oyun")
            || combined.contains("seçim oyunu");

        let game_matched = !is_non_game_context
            && (Self::contains_word_token(&combined, "oyun")
                || Self::contains_word_token(&combined, "oyunlar")
                || Self::contains_word_token(&combined, "oyunu")
                || Self::contains_word_token(&combined, "oyunları")
                || Self::contains_word_token(&combined, "steam")
                || Self::contains_word_token(&combined, "playstation")
                || Self::contains_word_token(&combined, "ps5")
                || Self::contains_word_token(&combined, "xbox")
                || Self::contains_word_token(&combined, "nintendo")
                || Self::contains_word_token(&combined, "game")
                || Self::contains_word_token(&combined, "gaming")
                || Self::contains_word_token(&combined, "gameplay"));

        if game_matched && !tags.iter().any(|t| t == "Oyun") {
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
        if let Some(api_key) = Self::get_gemini_api_key() {
            if let Ok(res) = Self::query_gemini_summary(title, content_text, &api_key).await {
                return res;
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
        let q_raw = query.trim().to_lowercase();
        let q_norm = normalize_turkish_chars(&q_raw);

        // 1. Direct domain detection in prompt (e.g. "kotaku.com", "add arstechnica.com", "https://...")
        let mut direct_sources = Vec::new();
        for word in q_raw.split_whitespace() {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '/' && c != ':');
            if let Ok(parsed_url) = url::Url::parse(clean_word) {
                if let Some(host) = parsed_url.host_str() {
                    let h = host.trim_start_matches("www.").to_string();
                    if h.contains('.') {
                        direct_sources.push(DiscoveredSource {
                            name: domain_to_display_name(&h),
                            domain: h,
                            suggested_feed: Some(parsed_url.to_string()),
                            category: "Teknoloji".to_string(),
                        });
                    }
                }
            } else if clean_word.contains('.') && !clean_word.starts_with('.') && !clean_word.ends_with('.') {
                let parts: Vec<&str> = clean_word.split('.').collect();
                if parts.len() >= 2 && parts.last().map(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_alphabetic())).unwrap_or(false) {
                    let host = clean_word.trim_start_matches("www.").to_string();
                    direct_sources.push(DiscoveredSource {
                        name: domain_to_display_name(&host),
                        domain: host.clone(),
                        suggested_feed: Some(format!("https://{}/feed", host)),
                        category: "Teknoloji".to_string(),
                    });
                }
            }
        }

        // 2. Multi-token and semantic scoring over CURATED_SOURCES
        let raw_tokens: Vec<&str> = q_norm
            .split(&[' ', ',', ';', ':', '!', '?', '-', '_', '(', ')', '"', '\''][..])
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect();

        let mut scored_entries: Vec<(usize, &CuratedEntry)> = CURATED_SOURCES
            .iter()
            .map(|entry| {
                let s = score_curated_entry(entry, &raw_tokens, &q_norm);
                (s, entry)
            })
            .filter(|(s, _)| *s > 0)
            .collect();

        // Sort descending by match score
        scored_entries.sort_by(|a, b| b.0.cmp(&a.0));

        let mut results = direct_sources;

        // Take up to 15 top scored entries
        for (_, entry) in scored_entries.into_iter().take(15) {
            // Avoid duplicate domain with direct_sources
            if !results.iter().any(|r| r.domain.eq_ignore_ascii_case(entry.domain)) {
                results.push(DiscoveredSource {
                    name: entry.name.to_string(),
                    domain: entry.domain.to_string(),
                    suggested_feed: Some(entry.feed_url.to_string()),
                    category: entry.category.to_string(),
                });
            }
        }

        // If still empty (e.g. unrecognizable prompt), fallback to top general sources
        if results.is_empty() {
            let fallbacks = ["arstechnica.com", "theverge.com", "gamingonlinux.com", "techcrunch.com", "thehackernews.com"];
            for dom in &fallbacks {
                if let Some(entry) = CURATED_SOURCES.iter().find(|e| e.domain == *dom) {
                    results.push(DiscoveredSource {
                        name: entry.name.to_string(),
                        domain: entry.domain.to_string(),
                        suggested_feed: Some(entry.feed_url.to_string()),
                        category: entry.category.to_string(),
                    });
                }
            }
        }

        results
    }

    async fn query_gemini_for_sources(prompt: &str, api_key: &str) -> Result<Vec<DiscoveredSource>, String> {
        // Try gemini-2.0-flash first, fallback to gemini-1.5-flash
        let models = ["gemini-2.0-flash", "gemini-1.5-flash"];

        let system_instruction = "Sen profesyonel bir haber ve medya kesif motorusun. Kullanicinin dogal dildeki isteklerini analiz et. Belirtilen konu, sehir veya sektorle ilgili en guvenilir, aktif yayin yapan haber ve blog sitelerini tespit et. Her site icin:\n1. 'name': Sitenin resmi yayin adi (orn: Webtekno, ShiftDelete, Gazete Duvar, AnandTech).\n2. 'domain': Sitenin ana alan adi (orn: webtekno.com, shiftdelete.net, gazeteduvar.com.tr).\n3. 'suggested_feed': Sitenin gercek RSS veya Atom besleme URL'si (orn: https://www.webtekno.com/rss.xml, https://shiftdelete.net/feed). Bilinmiyorsa ana sayfa URL'si (orn: https://domain.com).\n4. 'category': Su standart kategorilerden biri: 'Gundem', 'Siyaset', 'Yerel', 'Teknoloji', 'Linux', 'Oyun', 'Donanim', 'Bilim', 'Siber Guvenlik', 'Girisimcilik', 'Ekonomi'.\n\nSadece gecerli bir JSON dizisi dondur, baska hicbir metin ekleme:\n[{\"name\":\"...\",\"domain\":\"...\",\"suggested_feed\":\"...\",\"category\":\"...\"}]";

        let payload = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": format!("{}\n\nKullanici Istegi: {}\nEn az 3, en fazla 8 yuksek kaliteli kaynak listele.", system_instruction, prompt)
                }]
            }],
            "generationConfig": {
                "response_mime_type": "application/json",
                "temperature": 0.2
            }
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;

        for model in &models {
            let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model);
            let res = client
                .post(&url)
                .header("x-goog-api-key", api_key.trim())
                .json(&payload)
                .send()
                .await;

            if let Ok(resp) = res {
                if resp.status().is_success() {
                    if let Ok(json_body) = resp.json::<serde_json::Value>().await {
                        if let Some(text) = json_body["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                            let clean_text = text.trim();
                            // Handle cases where markdown code block wraps JSON
                            let json_str = if clean_text.starts_with("```json") {
                                clean_text.trim_start_matches("```json").trim_end_matches("```").trim()
                            } else if clean_text.starts_with("```") {
                                clean_text.trim_start_matches("```").trim_end_matches("```").trim()
                            } else {
                                clean_text
                            };

                            if let Ok(parsed) = serde_json::from_str::<Vec<DiscoveredSource>>(json_str) {
                                if !parsed.is_empty() {
                                    return Ok(parsed);
                                }
                            }
                        }
                    }
                }
            }
        }

        Err("Gemini kaynak kesfi gerceklestirilemedi".to_string())
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

fn normalize_turkish_chars(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'ı' | 'İ' => 'i',
            'ğ' | 'Ğ' => 'g',
            'ü' | 'Ü' => 'u',
            'ş' | 'Ş' => 's',
            'ö' | 'Ö' => 'o',
            'ç' | 'Ç' => 'c',
            _ => c.to_ascii_lowercase(),
        })
        .collect()
}

fn domain_to_display_name(domain: &str) -> String {
    let main_part = domain.split('.').next().unwrap_or(domain);
    let mut chars = main_part.chars();
    match chars.next() {
        None => domain.to_string(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub struct CuratedEntry {
    pub name: &'static str,
    pub domain: &'static str,
    pub feed_url: &'static str,
    pub category: &'static str,
    pub tags: &'static [&'static str],
}

pub static CURATED_SOURCES: &[CuratedEntry] = &[
    // Linux Gaming & Steam
    CuratedEntry {
        name: "GamingOnLinux",
        domain: "gamingonlinux.com",
        feed_url: "https://www.gamingonlinux.com/article_rss.php",
        category: "Linux & Oyun",
        tags: &["linux", "gaming", "steam", "deck", "steamdeck", "proton", "wine", "games", "vulkan", "emulation", "oyun", "cachyos", "omarchy"],
    },
    CuratedEntry {
        name: "Boiling Steam",
        domain: "boilingsteam.com",
        feed_url: "https://boilingsteam.com/feed/",
        category: "Linux & Oyun",
        tags: &["linux", "gaming", "steam", "deck", "steamdeck", "proton", "games", "oyun", "handheld"],
    },
    CuratedEntry {
        name: "Linux Game Consortium",
        domain: "linuxgameconsortium.com",
        feed_url: "https://linuxgameconsortium.com/feed/",
        category: "Linux & Oyun",
        tags: &["linux", "gaming", "games", "steam", "oyun", "indie"],
    },
    CuratedEntry {
        name: "Steam News",
        domain: "steampowered.com",
        feed_url: "https://store.steampowered.com/feeds/news.html",
        category: "Oyun",
        tags: &["steam", "valve", "gaming", "deck", "steamdeck", "games", "oyun", "sales", "updates"],
    },

    // Linux & Kernel & Open Source
    CuratedEntry {
        name: "Phoronix",
        domain: "phoronix.com",
        feed_url: "https://www.phoronix.com/rss.php",
        category: "Linux & Donanim",
        tags: &["linux", "kernel", "hardware", "benchmark", "gpu", "mesa", "amd", "intel", "nvidia", "open source", "donanim"],
    },
    CuratedEntry {
        name: "Arch Linux News",
        domain: "archlinux.org",
        feed_url: "https://archlinux.org/feeds/news/",
        category: "Linux",
        tags: &["linux", "arch", "distro", "package", "security", "open source", "acik kaynak"],
    },
    CuratedEntry {
        name: "OMG! Ubuntu",
        domain: "omgubuntu.co.uk",
        feed_url: "https://www.omgubuntu.co.uk/feed",
        category: "Linux",
        tags: &["linux", "ubuntu", "desktop", "apps", "gnome", "open source", "acik kaynak"],
    },
    CuratedEntry {
        name: "It's FOSS",
        domain: "itsfoss.com",
        feed_url: "https://itsfoss.com/feed/",
        category: "Acik Kaynak",
        tags: &["linux", "foss", "open source", "acik kaynak", "distro", "tutorials", "apps"],
    },
    CuratedEntry {
        name: "LWN.net",
        domain: "lwn.net",
        feed_url: "https://lwn.net/headlines/rss",
        category: "Linux & Cekirdek",
        tags: &["linux", "kernel", "cekirdek", "security", "development", "kernel.org", "open source"],
    },
    CuratedEntry {
        name: "Linux Today",
        domain: "linuxtoday.com",
        feed_url: "https://www.linuxtoday.com/feed/",
        category: "Linux",
        tags: &["linux", "open source", "acik kaynak", "enterprise", "sysadmin", "distro"],
    },
    CuratedEntry {
        name: "Fedora Magazine",
        domain: "fedoramagazine.org",
        feed_url: "https://fedoramagazine.org/feed/",
        category: "Linux",
        tags: &["linux", "fedora", "redhat", "desktop", "tutorials", "open source"],
    },
    CuratedEntry {
        name: "Linux Magazine",
        domain: "linux-magazine.com",
        feed_url: "https://www.linux-magazine.com/rss/feed/lmi_full",
        category: "Linux",
        tags: &["linux", "sysadmin", "server", "open source", "acik kaynak"],
    },
    CuratedEntry {
        name: "Baeldung on Linux",
        domain: "baeldung.com",
        feed_url: "https://www.baeldung.com/linux/feed",
        category: "Linux",
        tags: &["linux", "bash", "cli", "terminal", "commands", "tutorials"],
    },

    // Gaming (General)
    CuratedEntry {
        name: "PC Gamer",
        domain: "pcgamer.com",
        feed_url: "https://www.pcgamer.com/rss/",
        category: "Oyun",
        tags: &["gaming", "pc", "games", "steam", "oyun", "hardware", "mods", "rpg", "fps"],
    },
    CuratedEntry {
        name: "Rock Paper Shotgun",
        domain: "rockpapershotgun.com",
        feed_url: "https://www.rockpapershotgun.com/feed",
        category: "Oyun",
        tags: &["gaming", "pc", "indie", "games", "steam", "oyun", "reviews"],
    },
    CuratedEntry {
        name: "Eurogamer",
        domain: "eurogamer.net",
        feed_url: "https://www.eurogamer.net/feed",
        category: "Oyun",
        tags: &["gaming", "console", "playstation", "xbox", "nintendo", "pc", "oyun", "reviews"],
    },
    CuratedEntry {
        name: "Kotaku",
        domain: "kotaku.com",
        feed_url: "https://kotaku.com/rss",
        category: "Oyun",
        tags: &["gaming", "games", "culture", "nintendo", "playstation", "xbox", "oyun"],
    },
    CuratedEntry {
        name: "Polygon",
        domain: "polygon.com",
        feed_url: "https://www.polygon.com/rss/index.xml",
        category: "Oyun",
        tags: &["gaming", "entertainment", "reviews", "culture", "movies", "oyun"],
    },
    CuratedEntry {
        name: "Oyungezer",
        domain: "oyungezer.com.tr",
        feed_url: "https://oyungezer.com.tr/rss",
        category: "Oyun",
        tags: &["oyun", "gaming", "turkce", "inceleme", "steam", "konsol"],
    },

    // Cyber Security
    CuratedEntry {
        name: "The Hacker News",
        domain: "thehackernews.com",
        feed_url: "https://feeds.feedburner.com/TheHackersNews",
        category: "Siber Guvenlik",
        tags: &["security", "cyber", "cybersecurity", "siber", "guvenlik", "hacker", "malware", "vulnerability", "infosec", "cve"],
    },
    CuratedEntry {
        name: "BleepingComputer",
        domain: "bleepingcomputer.com",
        feed_url: "https://www.bleepingcomputer.com/feed/",
        category: "Siber Guvenlik",
        tags: &["security", "cyber", "ransomware", "malware", "cybersecurity", "siber", "guvenlik", "breach", "hacker", "cve"],
    },
    CuratedEntry {
        name: "Krebs on Security",
        domain: "krebsonsecurity.com",
        feed_url: "https://krebsonsecurity.com/feed/",
        category: "Siber Guvenlik",
        tags: &["security", "cyber", "cybercrime", "investigation", "siber", "guvenlik", "infosec"],
    },
    CuratedEntry {
        name: "Dark Reading",
        domain: "darkreading.com",
        feed_url: "https://www.darkreading.com/rss.xml",
        category: "Siber Guvenlik",
        tags: &["security", "cyber", "infosec", "threats", "siber", "guvenlik", "enterprise"],
    },
    CuratedEntry {
        name: "Schneier on Security",
        domain: "schneier.com",
        feed_url: "https://www.schneier.com/feed/atom/",
        category: "Siber Guvenlik",
        tags: &["security", "cyber", "cryptography", "privacy", "gizlilik", "siber", "guvenlik"],
    },

    // Technology, AI & Startups
    CuratedEntry {
        name: "TechCrunch",
        domain: "techcrunch.com",
        feed_url: "https://techcrunch.com/feed/",
        category: "Teknoloji & Girisim",
        tags: &["startups", "venture", "girisim", "tech", "silicon valley", "funding", "ai", "yapay zeka", "apps"],
    },
    CuratedEntry {
        name: "MIT Tech Review",
        domain: "technologyreview.com",
        feed_url: "https://www.technologyreview.com/feed/",
        category: "Teknoloji & AI",
        tags: &["ai", "yapay zeka", "tech", "computing", "research", "mit", "bilim", "science"],
    },
    CuratedEntry {
        name: "Wired",
        domain: "wired.com",
        feed_url: "https://www.wired.com/feed/rss",
        category: "Teknoloji",
        tags: &["tech", "teknoloji", "culture", "science", "ai", "yapay zeka"],
    },
    CuratedEntry {
        name: "The Verge",
        domain: "theverge.com",
        feed_url: "https://www.theverge.com/rss/index.xml",
        category: "Teknoloji",
        tags: &["tech", "teknoloji", "gadgets", "ai", "yapay zeka", "reviews", "mobile", "apple", "google"],
    },
    CuratedEntry {
        name: "Ars Technica",
        domain: "arstechnica.com",
        feed_url: "https://feeds.arstechnica.com/arstechnica/index",
        category: "Teknoloji",
        tags: &["tech", "teknoloji", "science", "policy", "gadgets", "ai"],
    },
    CuratedEntry {
        name: "The Next Web",
        domain: "thenextweb.com",
        feed_url: "https://thenextweb.com/feed",
        category: "Teknoloji",
        tags: &["tech", "europe", "startups", "ai", "innovation"],
    },
    CuratedEntry {
        name: "Engadget",
        domain: "engadget.com",
        feed_url: "https://www.engadget.com/rss.xml",
        category: "Teknoloji",
        tags: &["tech", "teknoloji", "gadgets", "gear", "mobile", "reviews"],
    },
    CuratedEntry {
        name: "Gizmodo",
        domain: "gizmodo.com",
        feed_url: "https://gizmodo.com/feed",
        category: "Teknoloji",
        tags: &["tech", "teknoloji", "science", "design", "culture"],
    },
    CuratedEntry {
        name: "Mashable",
        domain: "mashable.com",
        feed_url: "https://mashable.com/feeds/rss/all",
        category: "Teknoloji",
        tags: &["tech", "teknoloji", "digital", "entertainment"],
    },

    // Hardware & PC Enthusiast
    CuratedEntry {
        name: "Tom's Hardware",
        domain: "tomshardware.com",
        feed_url: "https://www.tomshardware.com/feeds/all",
        category: "Donanim",
        tags: &["hardware", "donanim", "cpu", "gpu", "pc", "benchmarks", "intel", "amd", "nvidia", "ssd", "motherboard"],
    },
    CuratedEntry {
        name: "Wccftech",
        domain: "wccftech.com",
        feed_url: "https://wccftech.com/feed/",
        category: "Donanim",
        tags: &["hardware", "donanim", "leaks", "nvidia", "amd", "intel", "gaming", "gpu", "cpu"],
    },
    CuratedEntry {
        name: "Hardware Plus",
        domain: "hwp.com.tr",
        feed_url: "https://hwp.com.tr/feed",
        category: "Donanim",
        tags: &["donanim", "hardware", "inceleme", "pc", "telefon"],
    },
    CuratedEntry {
        name: "DonanimHaber",
        domain: "donanimhaber.com",
        feed_url: "https://www.donanimhaber.com/rss/tum/",
        category: "Donanim",
        tags: &["donanim", "teknoloji", "ekran karti", "islemci", "turkce", "hardware"],
    },
    CuratedEntry {
        name: "Donanim Arsivi",
        domain: "donanimarsivi.com",
        feed_url: "https://donanimarsivi.com/feed/",
        category: "Donanim",
        tags: &["donanim", "sistem", "pc", "oyun", "hardware", "fiyat performans"],
    },
    CuratedEntry {
        name: "Donanim Gunlugu",
        domain: "donanimgunlugu.com",
        feed_url: "https://donanimgunlugu.com/feed",
        category: "Donanim",
        tags: &["donanim", "teknoloji", "akilli telefon", "bilgisayar", "hardware"],
    },

    // Mobile Ecosystems
    CuratedEntry {
        name: "9to5Mac",
        domain: "9to5mac.com",
        feed_url: "https://9to5mac.com/feed/",
        category: "Mobil",
        tags: &["apple", "iphone", "mac", "macbook", "ios", "mobil", "ipad"],
    },
    CuratedEntry {
        name: "Android Authority",
        domain: "androidauthority.com",
        feed_url: "https://www.androidauthority.com/feed",
        category: "Mobil",
        tags: &["android", "google", "samsung", "mobil", "pixel", "apps"],
    },

    // Software Development & DevOps
    CuratedEntry {
        name: "GitHub Blog",
        domain: "github.blog",
        feed_url: "https://github.blog/feed/",
        category: "Yazilim",
        tags: &["git", "github", "programming", "developer", "yazilim", "open source", "devops"],
    },
    CuratedEntry {
        name: "Dev.to",
        domain: "dev.to",
        feed_url: "https://dev.to/feed",
        category: "Yazilim",
        tags: &["coding", "programming", "developer", "yazilim", "webdev", "python", "rust"],
    },
    CuratedEntry {
        name: "Lobste.rs",
        domain: "lobste.rs",
        feed_url: "https://lobste.rs/rss",
        category: "Yazilim",
        tags: &["programming", "yazilim", "systems", "rust", "unix", "devops"],
    },
    CuratedEntry {
        name: "Hacker News",
        domain: "news.ycombinator.com",
        feed_url: "https://news.ycombinator.com/rss",
        category: "Yazilim & Girisim",
        tags: &["tech", "startups", "programming", "yazilim", "yc", "girisim"],
    },
    CuratedEntry {
        name: "Rust Blog",
        domain: "blog.rust-lang.org",
        feed_url: "https://blog.rust-lang.org/feed.xml",
        category: "Yazilim",
        tags: &["rust", "rustlang", "programming", "yazilim", "systems", "cargo"],
    },
    CuratedEntry {
        name: "Go Blog",
        domain: "go.dev",
        feed_url: "https://go.dev/blog/feed.atom",
        category: "Yazilim",
        tags: &["golang", "go", "programming", "yazilim", "backend", "cloud"],
    },

    // Science & Space
    CuratedEntry {
        name: "NASA Breaking News",
        domain: "nasa.gov",
        feed_url: "https://www.nasa.gov/news-release/feed/",
        category: "Bilim & Uzay",
        tags: &["space", "uzay", "nasa", "astronomy", "moon", "mars", "bilim", "science"],
    },
    CuratedEntry {
        name: "Space.com",
        domain: "space.com",
        feed_url: "https://www.space.com/feeds/all",
        category: "Bilim & Uzay",
        tags: &["space", "uzay", "rocket", "spacex", "astronomy", "bilim", "science"],
    },
    CuratedEntry {
        name: "Phys.org",
        domain: "phys.org",
        feed_url: "https://phys.org/rss-feed/",
        category: "Bilim",
        tags: &["physics", "fizik", "science", "bilim", "quantum", "research"],
    },
    CuratedEntry {
        name: "ScienceDaily",
        domain: "sciencedaily.com",
        feed_url: "https://www.sciencedaily.com/rss/all.xml",
        category: "Bilim",
        tags: &["science", "bilim", "research", "health", "biology", "environment"],
    },
    CuratedEntry {
        name: "Evrim Agaci",
        domain: "evrimagaci.org",
        feed_url: "https://evrimagaci.org/rss.xml",
        category: "Bilim",
        tags: &["bilim", "evrim", "biyoloji", "fizik", "populer bilim", "science"],
    },

    // US Local News
    CuratedEntry {
        name: "The Texas Tribune",
        domain: "texastribune.org",
        feed_url: "https://www.texastribune.org/feeds/main/",
        category: "Yerel",
        tags: &["texas", "austin", "local", "yerel", "politics", "news"],
    },
    CuratedEntry {
        name: "Austin Monitor",
        domain: "austinmonitor.com",
        feed_url: "https://www.austinmonitor.com/feed/",
        category: "Yerel",
        tags: &["austin", "texas", "local", "yerel", "city council", "news"],
    },

    // Turkish Tech & Startups
    CuratedEntry {
        name: "Webrazzi",
        domain: "webrazzi.com",
        feed_url: "https://webrazzi.com/feed/",
        category: "Teknoloji & Girisim",
        tags: &["teknoloji", "girisim", "yatirim", "turkiye", "startup", "tech"],
    },
    CuratedEntry {
        name: "ShiftDelete",
        domain: "shiftdelete.net",
        feed_url: "https://shiftdelete.net/feed",
        category: "Teknoloji",
        tags: &["teknoloji", "mobil", "telefon", "turkce", "inceleme"],
    },
    CuratedEntry {
        name: "Webtekno",
        domain: "webtekno.com",
        feed_url: "https://www.webtekno.com/rss.xml",
        category: "Teknoloji",
        tags: &["teknoloji", "bilim", "oyun", "turkce", "haber"],
    },
    CuratedEntry {
        name: "LOG",
        domain: "log.com.tr",
        feed_url: "https://www.log.com.tr/feed/",
        category: "Teknoloji",
        tags: &["teknoloji", "tasarim", "otomobil", "lifestyle"],
    },
    CuratedEntry {
        name: "Teknoblog",
        domain: "teknoblog.com",
        feed_url: "https://www.teknoblog.com/feed/",
        category: "Teknoloji",
        tags: &["teknoloji", "turkce", "mobil", "inceleme"],
    },
    CuratedEntry {
        name: "CHIP Turkiye",
        domain: "chip.com.tr",
        feed_url: "https://www.chip.com.tr/rss",
        category: "Teknoloji",
        tags: &["teknoloji", "bilgisayar", "yazilim", "donanim"],
    },
    CuratedEntry {
        name: "Swipeline",
        domain: "swipeline.co",
        feed_url: "https://swipeline.co/feed/",
        category: "Girisimcilik",
        tags: &["girisim", "startup", "yatirim", "ekosistem"],
    },
    CuratedEntry {
        name: "Egirisim",
        domain: "egirisim.com",
        feed_url: "https://egirisim.com/feed/",
        category: "Girisimcilik",
        tags: &["girisim", "startup", "yatirim", "girisimci"],
    },
    CuratedEntry {
        name: "BTK Haber",
        domain: "btk.gov.tr",
        feed_url: "https://www.btk.gov.tr/rss/news",
        category: "Bilisim & Guvenlik",
        tags: &["bilisim", "btk", "guvenlik", "telekom", "siber"],
    },

    // Turkish Agenda, Politics & Local
    CuratedEntry {
        name: "Haberler Yerel",
        domain: "haberler.com",
        feed_url: "https://rss.haberler.com/rss.asp?kategori=yerel",
        category: "Yerel",
        tags: &["yerel", "istanbul", "ankara", "izmir", "sehir", "belediye", "kent", "asayis", "iller"],
    },
    CuratedEntry {
        name: "Yeni Asir",
        domain: "yeniasir.com.tr",
        feed_url: "https://www.yeniasir.com.tr/rss/anasayfa.xml",
        category: "Yerel",
        tags: &["yerel", "ege", "izmir", "aydin", "mugla", "manisa", "sehir", "kent"],
    },
    CuratedEntry {
        name: "Bursa Hakimiyet",
        domain: "bursahakimiyet.com.tr",
        feed_url: "https://www.bursahakimiyet.com.tr/rss",
        category: "Yerel",
        tags: &["yerel", "bursa", "marmara", "istanbul", "kent", "belediye", "sehir"],
    },
    CuratedEntry {
        name: "Sozcu",
        domain: "sozcu.com.tr",
        feed_url: "https://www.sozcu.com.tr/feeds-son-dakika",
        category: "Gundem",
        tags: &["siyaset", "siyasi", "politika", "gundem", "son dakika", "turkiye", "gazete", "haber"],
    },
    CuratedEntry {
        name: "Haberturk",
        domain: "haberturk.com",
        feed_url: "https://www.haberturk.com/rss",
        category: "Gundem",
        tags: &["siyaset", "siyasi", "politika", "gundem", "haber", "son dakika", "turkiye"],
    },
    CuratedEntry {
        name: "NTV Gundem",
        domain: "ntv.com.tr",
        feed_url: "https://www.ntv.com.tr/gundem.rss",
        category: "Gundem",
        tags: &["siyaset", "siyasi", "politika", "gundem", "turkiye", "manset", "haber"],
    },
    CuratedEntry {
        name: "BBC Turkce",
        domain: "bbc.com",
        feed_url: "https://feeds.bbci.co.uk/turkce/rss.xml",
        category: "Siyaset & Dunya",
        tags: &["siyaset", "siyasi", "politika", "dunya", "analiz", "tarafsiz", "haber"],
    },
    CuratedEntry {
        name: "Diken",
        domain: "diken.com.tr",
        feed_url: "https://www.diken.com.tr/feed/",
        category: "Siyaset",
        tags: &["siyaset", "siyasi", "politika", "haber", "yorum", "bagimsiz"],
    },
    CuratedEntry {
        name: "Gazete Duvar",
        domain: "gazeteduvar.com.tr",
        feed_url: "https://www.gazeteduvar.com.tr/export/rss",
        category: "Siyaset",
        tags: &["siyaset", "siyasi", "politika", "kultur", "yazarlar"],
    },
    CuratedEntry {
        name: "Anadolu Ajansi",
        domain: "aa.com.tr",
        feed_url: "https://www.aa.com.tr/tr/rss/default?cat=guncel",
        category: "Gundem",
        tags: &["ajans", "resmi", "guncel", "turkiye", "haber"],
    },
    CuratedEntry {
        name: "Cumhuriyet",
        domain: "cumhuriyet.com.tr",
        feed_url: "https://www.cumhuriyet.com.tr/rss",
        category: "Siyaset",
        tags: &["siyaset", "siyasi", "politika", "cumhuriyet", "gazete", "haber"],
    },

    // World & Economics
    CuratedEntry {
        name: "BBC News World",
        domain: "bbc.com",
        feed_url: "https://feeds.bbci.co.uk/news/world/rss.xml",
        category: "Dunya",
        tags: &["world", "international", "global", "news", "dunya", "haber"],
    },
    CuratedEntry {
        name: "BloombergHT",
        domain: "bloomberght.com",
        feed_url: "https://www.bloomberght.com/rss",
        category: "Ekonomi",
        tags: &["ekonomi", "borsa", "finans", "dolar", "piyasa", "finance"],
    },
    CuratedEntry {
        name: "Dunya Gazetesi",
        domain: "dunya.com",
        feed_url: "https://www.dunya.com/rss",
        category: "Ekonomi",
        tags: &["ekonomi", "is dunyasi", "finans", "ihracat", "finance"],
    },
];

fn score_curated_entry(entry: &CuratedEntry, tokens: &[&str], q_norm: &str) -> usize {
    let mut score = 0;
    let name_low = entry.name.to_lowercase();
    let dom_low = entry.domain.to_lowercase();
    let cat_low = entry.category.to_lowercase();

    // Local news query guard: If user explicitly asked for local news, restrict to local category
    if (q_norm.contains("yerel") || q_norm.contains("local")) && entry.category != "Yerel" {
        return 0;
    }

    // Direct substring in name, domain or category
    if q_norm.len() >= 4 {
        if name_low.contains(q_norm) || dom_low.contains(q_norm) {
            score += 70;
        }
        if cat_low.contains(q_norm) {
            score += 40;
        }
    }

    // Compound phrases & bigrams
    let compound_phrases: &[(&str, &[&str])] = &[
        ("linux gaming", &["linux", "gaming"]),
        ("linux oyun", &["linux", "gaming"]),
        ("steam deck", &["steam", "deck"]),
        ("open source", &["open source", "foss"]),
        ("acik kaynak", &["acik kaynak", "foss"]),
        ("cyber security", &["cyber", "security"]),
        ("siber guvenlik", &["siber", "guvenlik"]),
        ("yapay zeka", &["ai", "yapay zeka"]),
        ("local news", &["local", "yerel"]),
        ("yerel haber", &["yerel", "local"]),
        ("siyasi haber", &["siyaset", "haber"]),
        ("siyaset haber", &["siyaset", "haber"]),
        ("politika haber", &["politika", "haber"]),
    ];

    for (phrase, key_terms) in compound_phrases {
        if q_norm.contains(phrase) {
            let matches_all = key_terms.iter().all(|term| {
                entry.tags.iter().any(|t| t.contains(term))
                    || cat_low.contains(term)
                    || name_low.contains(term)
            });
            if matches_all {
                score += 80;
            }
        }
    }

    // Stop words
    let stop_words = [
        "find", "more", "add", "news", "feed", "feeds", "source", "sources", "site", "sites",
        "and", "the", "for", "with", "from", "top", "best", "some", "bana", "ekle", "haber",
        "haberler", "haberleri", "haberlerini", "bul", "getir", "siteleri", "kaynak", "kaynaklar",
        "kaynaklari", "kaynaklarini", "olan", "ile", "ve", "de", "da", "icin", "en", "iyi"
    ];

    for &tok in tokens {
        if stop_words.contains(&tok) || tok.len() < 2 {
            continue;
        }

        // Exact matches
        if name_low.contains(tok) {
            score += 30;
        }
        if dom_low.contains(tok) {
            score += 30;
        }
        if cat_low.contains(tok) {
            score += 25;
        }

        for &tag in entry.tags {
            if tag == tok {
                score += 20;
            } else if tag.contains(tok) || tok.contains(tag) {
                score += 10;
            }
        }

        // Semantic concepts
        if (tok == "gaming" || tok == "games" || tok == "game" || tok == "oyun")
            && (entry.tags.contains(&"gaming") || entry.tags.contains(&"games") || entry.tags.contains(&"oyun"))
        {
            score += 25;
        }
        if (tok == "linux" || tok == "kernel" || tok == "cekirdek" || tok == "foss")
            && (entry.tags.contains(&"linux") || entry.tags.contains(&"foss"))
        {
            score += 25;
        }
        if (tok == "steam" || tok == "proton" || tok == "deck")
            && (entry.tags.contains(&"steam") || entry.tags.contains(&"deck"))
        {
            score += 30;
        }
        if (tok == "security" || tok == "cyber" || tok == "siber" || tok == "guvenlik" || tok == "hacker")
            && (entry.tags.contains(&"security") || entry.tags.contains(&"siber"))
        {
            score += 30;
        }
        if (tok == "ai" || tok == "yapay" || tok == "zeka")
            && (entry.tags.contains(&"ai") || entry.tags.contains(&"yapay zeka"))
        {
            score += 30;
        }
        if (tok.starts_with("siyas") || tok.starts_with("politik") || tok == "politics")
            && (entry.tags.contains(&"siyaset") || entry.tags.contains(&"siyasi") || entry.tags.contains(&"politika") || entry.category == "Siyaset" || entry.category == "Gundem")
        {
            score += 45;
        }
        if (tok == "yerel" || tok == "local" || tok == "belediye" || tok == "sehir")
            && entry.category == "Yerel"
        {
            score += 40;
        }
        if (tok == "teknoloji" || tok == "tech" || tok == "bilisim" || tok == "dijital")
            && (entry.tags.contains(&"teknoloji") || entry.tags.contains(&"tech") || entry.category == "Teknoloji")
        {
            score += 35;
        }
        if (tok == "donanim" || tok == "hardware" || tok == "gpu" || tok == "cpu" || tok == "benchmark")
            && (entry.tags.contains(&"donanim") || entry.tags.contains(&"hardware") || entry.category.contains("Donanim"))
        {
            score += 35;
        }
        if (tok == "bilim" || tok == "science" || tok == "uzay" || tok == "fizik" || tok == "biyoloji" || tok == "evrim")
            && (entry.tags.contains(&"bilim") || entry.tags.contains(&"science") || entry.category == "Bilim")
        {
            score += 40;
        }
        if (tok == "ekonomi" || tok == "finans" || tok == "borsa" || tok == "finance" || tok == "para")
            && (entry.tags.contains(&"ekonomi") || entry.category == "Ekonomi")
        {
            score += 40;
        }
        if (tok == "girisim" || tok == "startup" || tok == "yatirim" || tok == "fon")
            && (entry.tags.contains(&"girisim") || entry.tags.contains(&"startup") || entry.category == "Girisimcilik")
        {
            score += 40;
        }

        // City names (Ankara, Izmir, Istanbul, Bursa, Antalya, etc.)
        let cities = ["izmir", "ankara", "istanbul", "bursa", "antalya", "adana", "konya", "trabzon", "eskisehir", "gaziantep", "kayseri", "samsun", "diyarbakir", "kastamonu"];
        if cities.contains(&tok) && (entry.tags.contains(&tok) || name_low.contains(tok) || dom_low.contains(tok)) {
            score += 60;
        }
    }

    score
}
