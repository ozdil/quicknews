use quicknews_core::core::adblock::AdBlocker;
use quicknews_core::core::ai::AiEngine;
use quicknews_core::core::extractor::ArticleExtractor;
use quicknews_core::core::feed::FeedParser;
use quicknews_core::core::security::{
    atomic_write_file, is_private_or_reserved_ipv4, safe_read_file, validate_url_ssrf,
};
use std::net::Ipv4Addr;

#[test]
fn test_ssrf_blocking_private_ips() {
    // Loopback
    assert!(is_private_or_reserved_ipv4(Ipv4Addr::new(127, 0, 0, 1)));
    // RFC 1918 Private ranges
    assert!(is_private_or_reserved_ipv4(Ipv4Addr::new(10, 0, 1, 5)));
    assert!(is_private_or_reserved_ipv4(Ipv4Addr::new(172, 16, 0, 1)));
    assert!(is_private_or_reserved_ipv4(Ipv4Addr::new(172, 31, 255, 254)));
    assert!(is_private_or_reserved_ipv4(Ipv4Addr::new(192, 168, 1, 1)));
    // Cloud metadata / link local
    assert!(is_private_or_reserved_ipv4(Ipv4Addr::new(169, 254, 169, 254)));

    // Public IPs should NOT be blocked
    assert!(!is_private_or_reserved_ipv4(Ipv4Addr::new(1, 1, 1, 1)));
    assert!(!is_private_or_reserved_ipv4(Ipv4Addr::new(8, 8, 8, 8)));
}

#[test]
fn test_url_ssrf_validator() {
    // Bad protocols
    assert!(validate_url_ssrf("file:///etc/passwd").is_err());
    assert!(validate_url_ssrf("ftp://example.com/test").is_err());
    assert!(validate_url_ssrf("gopher://example.com/").is_err());

    // Internal domains and IPs
    assert!(validate_url_ssrf("http://localhost:8080/secret").is_err());
    assert!(validate_url_ssrf("http://127.0.0.1/admin").is_err());
    assert!(validate_url_ssrf("http://169.254.169.254/latest/meta-data").is_err());
    assert!(validate_url_ssrf("http://192.168.1.100/router").is_err());

    // Valid public URLs
    assert!(validate_url_ssrf("https://webrazzi.com/feed/").is_ok());
    assert!(validate_url_ssrf("https://www.phoronix.com/rss.php").is_ok());
}

#[test]
fn test_ai_sentence_spacing_fix() {
    let input = "Devlet başkanı şirketi suçladı.Çin tarafı ise tepki gösterdi.Yeni karar alındı!Türkiye onayladı.";
    let fixed = AiEngine::fix_sentence_spacing(input);
    assert_eq!(
        fixed,
        "Devlet başkanı şirketi suçladı. Çin tarafı ise tepki gösterdi. Yeni karar alındı! Türkiye onayladı."
    );
}

#[test]
fn test_ai_heading_spacing_rule() {
    let text = "Ilk paragraf metni burada yer aliyor.\n\n## Alt Baslik\n\nIkinci paragraf metni basliktan sonra geliyor.";
    let cleaned = AiEngine::clean_blocks(text);
    // Basliktan evvel mutlaka en az uc yeni satir (\n\n\n##) bulunmali
    assert!(cleaned.contains("\n\n\n## Alt Baslik"));
}

#[test]
fn test_ai_auto_classify_tags() {
    let title = "Nvidia yeni nesil RTX 5090 ekran kartını ve yapay zeka çiplerini duyurdu";
    let text = "GPU mimarisi ve makine öğrenimi modelleri üzerinde olağanüstü performans sergiliyor.";
    let tags = AiEngine::auto_classify_tags("DonanımHaber", "Teknoloji", title, text);

    assert!(tags.contains(&"Teknoloji".to_string()));
    assert!(tags.contains(&"Yapay Zeka".to_string()));
    assert!(tags.contains(&"Donanım".to_string()));
}

#[test]
fn test_ai_tag_classification_linux() {
    let title = "Linux 6.14 çekirdeği yayınlandı: Bcachefs ve Omarchy geliştirmeleri";
    let text = "Yeni açık kaynak kernel güncellemesi performans artışları sunuyor.";
    let tags = AiEngine::auto_classify_tags("Phoronix", "Linux & Donanim", title, text);

    assert!(tags.contains(&"Linux & Donanim".to_string()));
    assert!(tags.contains(&"Açık Kaynak".to_string()));
}

#[test]
fn test_adblock_and_clutter_filters() {
    assert!(AdBlocker::is_forbidden_tag("script"));
    assert!(AdBlocker::is_forbidden_tag("iframe"));
    assert!(AdBlocker::is_forbidden_tag("img"));
    assert!(AdBlocker::is_forbidden_tag("style"));

    assert!(AdBlocker::is_ad_or_clutter_attribute("google-ad-banner"));
    assert!(AdBlocker::is_ad_or_clutter_attribute("newsletter-signup-box"));
    assert!(AdBlocker::is_ad_or_clutter_attribute("check-this-out related listing"));
    assert!(AdBlocker::is_ad_or_clutter_attribute("share-social-buttons"));

    assert!(!AdBlocker::is_ad_or_clutter_attribute("main-article-content"));
}

#[test]
fn test_article_extractor_clean_html() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Haber Basligi - Haber Sitesi</title>
            <meta name="author" content="Ahmet Yilmaz">
        </head>
        <body>
            <header><nav>Menu links</nav></header>
            <div class="ad-banner">Reklam alani 300x250</div>
            <article class="entry-content">
                <h1>Test Haber Basligi</h1>
                <p>Bu birinci paragraf metnidir ve haber hakkinda onemli bilgiler icermektedir.</p>
                <div class="social-share">Paylas facebook twitter</div>
                <p>Bu da ikinci paragraftir ve haberin detaylarini tam olarak aciklamaktadir.</p>
                <ul>
                    <li>Birinci onemli liste maddesi</li>
                    <li>Ikinci onemli liste maddesi</li>
                </ul>
            </article>
            <footer>Telif haklari 2026</footer>
        </body>
        </html>
    "#;

    let clean = ArticleExtractor::extract(html, "https://example.com/haber-1");
    assert_eq!(clean.title, "Test Haber Basligi");
    assert_eq!(clean.author, Some("Ahmet Yilmaz".to_string()));
    assert!(clean.content_text.contains("Bu birinci paragraf metnidir"));
    assert!(clean.content_text.contains("Bu da ikinci paragraftir"));
    assert!(clean.content_text.contains("- Birinci onemli liste maddesi"));
    assert!(!clean.content_text.contains("Reklam alani"));
    assert!(!clean.content_text.contains("Paylas facebook"));
}

#[test]
fn test_rss_xml_parsing() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <rss version="2.0">
        <channel>
            <title>Test Teknoloji Akisi</title>
            <item>
                <title>Yapay Zeka Gelismeleri 2026</title>
                <link>https://example.com/ai-2026?utm_source=rss</link>
                <description><![CDATA[<p>Yapay zeka modelleri hizla gelismeye devam ediyor.</p>]]></description>
                <pubDate>Fri, 18 Sep 2026 08:00:00 +0300</pubDate>
            </item>
        </channel>
    </rss>"#;

    let items = FeedParser::parse_xml(xml, "src_test", "Test Kaynak", "Teknoloji").unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Yapay Zeka Gelismeleri 2026");
    // UTM tracking should be stripped
    assert_eq!(items[0].link, "https://example.com/ai-2026");
    assert!(items[0].summary.contains("Yapay zeka modelleri hizla gelismeye devam ediyor"));
    assert!(items[0].tags.contains(&"Teknoloji".to_string()));
    assert!(items[0].tags.contains(&"Yapay Zeka".to_string()));
}

#[test]
fn test_atomic_file_write_and_safe_read() {
    let temp_dir = std::env::temp_dir().join(format!("quicknews_test_{}", std::process::id()));
    let file_path = temp_dir.join("test_data.json");
    let sample_bytes = b"{\"status\":\"OK\",\"version\":\"0.1.0\"}";

    let write_res = atomic_write_file(&file_path, sample_bytes);
    assert!(write_res.is_ok());

    let read_res = safe_read_file(&file_path, 1024);
    assert!(read_res.is_ok());
    assert_eq!(read_res.unwrap(), sample_bytes);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
