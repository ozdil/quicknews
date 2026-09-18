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

#[test]
fn test_benchmarking_and_advanced_ipv6_ssrf() {
    use quicknews_core::core::security::{is_private_or_reserved_ipv4, is_private_or_reserved_ipv6};
    use std::net::{Ipv4Addr, Ipv6Addr};

    // RFC 2544 Benchmarking
    assert!(is_private_or_reserved_ipv4(Ipv4Addr::new(198, 18, 0, 1)));
    assert!(is_private_or_reserved_ipv4(Ipv4Addr::new(198, 19, 255, 254)));

    // IPv6 NAT64 (RFC 6052) with embedded private/loopback IPv4
    let nat64_loopback = "64:ff9b::127.0.0.1".parse::<Ipv6Addr>().unwrap();
    assert!(is_private_or_reserved_ipv6(nat64_loopback));

    let nat64_private = "64:ff9b::192.168.1.1".parse::<Ipv6Addr>().unwrap();
    assert!(is_private_or_reserved_ipv6(nat64_private));

    // IPv6 6to4 (RFC 3056) with embedded private IPv4
    let six_to_four_loopback = "2002:7f00:0001::".parse::<Ipv6Addr>().unwrap();
    assert!(is_private_or_reserved_ipv6(six_to_four_loopback));

    // IPv6 Documentation & Benchmarking & Discard
    let doc_ipv6 = "2001:db8::1".parse::<Ipv6Addr>().unwrap();
    assert!(is_private_or_reserved_ipv6(doc_ipv6));

    let bench_ipv6 = "2001:2::1".parse::<Ipv6Addr>().unwrap();
    assert!(is_private_or_reserved_ipv6(bench_ipv6));

    let discard_ipv6 = "100::1".parse::<Ipv6Addr>().unwrap();
    assert!(is_private_or_reserved_ipv6(discard_ipv6));
}

#[test]
fn test_port_allowlist() {
    // Prohibited ports (SSH, Redis, SMTP)
    assert!(validate_url_ssrf("http://127.0.0.1:22/").is_err());
    assert!(validate_url_ssrf("http://127.0.0.1:6379/").is_err());
    assert!(validate_url_ssrf("http://127.0.0.1:25/").is_err());
    assert!(validate_url_ssrf("http://127.0.0.1:3306/").is_err());
}

#[test]
fn test_export_path_traversal_guards() {
    use quicknews_core::core::extractor::CleanArticle;
    use quicknews_core::core::storage::StorageManager;
    use std::path::Path;

    let storage = StorageManager::new();
    let sample_article = CleanArticle {
        title: "Guvenlik Testi".to_string(),
        author: None,
        published_date: None,
        source_url: "https://example.com/sec-test".to_string(),
        source_name: None,
        category: None,
        tags: vec![],
        content_text: "Test icerigi".to_string(),
        word_count: 2,
        reading_time_mins: 1,
    };

    // Reject parent dir traversal
    let bad_parent = storage.export_article_markdown(&sample_article, Some(Path::new("../../etc/shadow")));
    assert!(bad_parent.is_err());

    // Reject system directories
    let bad_etc = storage.export_article_markdown(&sample_article, Some(Path::new("/etc/crontab")));
    assert!(bad_etc.is_err());

    let bad_bin = storage.export_article_markdown(&sample_article, Some(Path::new("/bin/malicious.md")));
    assert!(bad_bin.is_err());

    // Reject non-markdown extension
    let bad_ext = storage.export_article_markdown(&sample_article, Some(Path::new("/tmp/test.exe")));
    assert!(bad_ext.is_err());
}

#[test]
fn test_rss_xml_parsing_with_malicious_urls() {
    let xml_with_evil_links = r#"<?xml version="1.0" encoding="UTF-8"?>
    <rss version="2.0">
        <channel>
            <title>Saldirgan Akisi</title>
            <item>
                <title>Hileli Javascript</title>
                <link>javascript:alert(document.cookie)</link>
                <description>Zararli betik baglantisi</description>
            </item>
            <item>
                <title>Yerel Dosya Saldirisi</title>
                <link>file:///etc/passwd</link>
                <description>Sistem dosyasi baglantisi</description>
            </item>
            <item>
                <title>Dahili Port Saldirisi</title>
                <link>http://localhost:8080/admin</link>
                <description>SSRF baglantisi</description>
            </item>
            <item>
                <title>Cloud Metadata Saldirisi</title>
                <link>http://169.254.169.254/latest/meta-data</link>
                <description>Cloud metadata baglantisi</description>
            </item>
            <item>
                <title>Gecerli Guvenli Haber</title>
                <link>https://example.com/haber-guvenli</link>
                <description>Bu haber guvenli bir baglantiya sahiptir.</description>
            </item>
        </channel>
    </rss>"#;

    let items = FeedParser::parse_xml(xml_with_evil_links, "src_evil", "Zararli", "Teknoloji").unwrap();
    // Tum zararli baglantilar elenmeli, yalnizca 1 adet gecerli ve guvenli haber kalmali!
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Gecerli Guvenli Haber");
    assert_eq!(items[0].link, "https://example.com/haber-guvenli");
}

#[test]
fn test_symlink_nofollow_rejection() {
    let temp_dir = std::env::temp_dir().join(format!("quicknews_symtest_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let real_file = temp_dir.join("real_target.txt");
    let _ = std::fs::write(&real_file, b"gizli icerik");

    let symlink_file = temp_dir.join("symlink_pointer.txt");
    let _ = std::os::unix::fs::symlink(&real_file, &symlink_file);

    // Reading a symlink MUST fail with SymlinkForbidden
    let read_res = safe_read_file(&symlink_file, 1024);
    assert!(read_res.is_err());

    // Writing to a symlink MUST fail with SymlinkForbidden
    let write_res = atomic_write_file(&symlink_file, b"yeni veri");
    assert!(write_res.is_err());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_broken_symlink_rejection() {
    let temp_dir = std::env::temp_dir().join(format!("quicknews_broken_symtest_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let nonexistent_target = temp_dir.join("does_not_exist.txt");
    let broken_symlink = temp_dir.join("broken_symlink.txt");
    let _ = std::os::unix::fs::symlink(&nonexistent_target, &broken_symlink);

    // Both reading and writing to a dangling/broken symlink MUST be rejected as SymlinkForbidden
    let read_res = safe_read_file(&broken_symlink, 1024);
    assert!(read_res.is_err());

    let write_res = atomic_write_file(&broken_symlink, b"guvensiz yazim");
    assert!(write_res.is_err());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_feed_item_flooding_dos_guard() {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss version=\"2.0\"><channel><title>Sel Testi</title>");
    for i in 0..150 {
        xml.push_str(&format!(
            "<item><title>Haber {}</title><link>https://example.com/item/{}</link><description>Aciklama</description></item>",
            i, i
        ));
    }
    xml.push_str("</channel></rss>");

    let items = FeedParser::parse_xml(&xml, "src_flood", "Sel Kaynagi", "Teknoloji").unwrap();
    // 150 ogelik besleme en fazla 100 oge ile sinirlandirilmalidir
    assert_eq!(items.len(), 100);
}

#[test]
fn test_html_extractor_dom_depth_and_block_limits() {
    // 50 seviye derinlikte ic ice gecmis HTML (derinlik > 32)
    let mut deep_html = String::new();
    for _ in 0..50 {
        deep_html.push_str("<div>");
    }
    deep_html.push_str("<p>Cok derindeki metin paragrafi burada bulunuyor ve ayiklanmamali.</p>");
    for _ in 0..50 {
        deep_html.push_str("</div>");
    }

    let article = ArticleExtractor::extract(&deep_html, "https://example.com/deep");
    // Derinlik 32'yi astigi icin icerik yigin tasmasina yol acmadan guvenle sonlanmali
    assert!(!article.content_text.contains("Cok derindeki metin paragrafi"));
}

#[test]
fn test_url_userinfo_rejection() {
    use quicknews_core::core::security::validate_url_ssrf_and_resolve;
    // URL containing credentials (user:pass@host) must be rejected
    assert!(validate_url_ssrf_and_resolve("http://admin:secret@example.com/test").is_err());
    assert!(validate_url_ssrf_and_resolve("https://user@example.com/path").is_err());
}

#[test]
fn test_storage_add_source_sanitization() {
    use quicknews_core::core::storage::StorageManager;
    let storage = StorageManager::new();

    // CRLF injection attempt
    let crlf_res = storage.add_source("Kaynak\nInjected", "injected.com", "https://injected.com/feed", "Teknoloji");
    assert!(crlf_res.is_err());

    // Excessively long parameter (> 100 chars)
    let long_name = "A".repeat(105);
    let long_res = storage.add_source(&long_name, "valid.com", "https://valid.com/feed", "Teknoloji");
    assert!(long_res.is_err());

    // Private IP SSRF attempt in feed URL
    let ssrf_res = storage.add_source("SSRF Kaynak", "192.168.1.1", "http://192.168.1.1/feed", "Teknoloji");
    assert!(ssrf_res.is_err());
}

#[test]
fn test_strip_markdown_images_beacon_filter() {
    let input = "Bu haber metnidir. ![Takip Pikseli](https://tracker.analytics.com/beacon.gif) Burada baska bir paragraf var. ![banner](https://ads.com/ad.jpg?user=123) Sonuc metni.";
    let stripped = AdBlocker::strip_markdown_images(input);
    assert!(!stripped.contains("https://tracker.analytics.com/beacon.gif"));
    assert!(!stripped.contains("https://ads.com/ad.jpg"));
    assert!(stripped.contains("Bu haber metnidir."));
    assert!(stripped.contains("Burada baska bir paragraf var."));
    assert!(stripped.contains("Sonuc metni."));
}

#[test]
fn test_symlink_prune_cache_guard() {
    use quicknews_core::core::storage::StorageManager;
    let temp_cache = std::env::temp_dir().join(format!("quicknews_cache_symtest_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_cache);

    // Target secret file outside cache
    let secret_file = std::env::temp_dir().join(format!("quicknews_outside_secret_{}.txt", std::process::id()));
    let _ = std::fs::write(&secret_file, b"gizli ve silinmemesi gereken sistem verisi");

    // Infiltrated symlink inside cache pointing to secret
    let symlink_in_cache = temp_cache.join("cached_article_symlink.json");
    let _ = std::os::unix::fs::symlink(&secret_file, &symlink_in_cache);

    // Run cache pruning
    StorageManager::prune_cache_directory(&temp_cache, 0);

    // Symlink in cache must be purged directly
    assert!(std::fs::symlink_metadata(&symlink_in_cache).is_err());
    // External target file must remain completely untouched and intact
    assert!(secret_file.exists());
    assert_eq!(std::fs::read(&secret_file).unwrap(), b"gizli ve silinmemesi gereken sistem verisi");

    let _ = std::fs::remove_file(&secret_file);
    let _ = std::fs::remove_dir_all(&temp_cache);
}

