use quicknews_core::core::ai::AiEngine;
use quicknews_core::QuickNewsApp;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let app = QuickNewsApp::new();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "sync" => {
            let is_json = args.iter().any(|a| a == "--json");
            let articles = app.sync_all_feeds().await;
            if is_json {
                println!("{}", serde_json::to_string(&articles).unwrap_or_default());
            } else {
                println!("Basariyla {} haber senkronize edildi.", articles.len());
            }
        }
        "list" => {
            let is_json = args.iter().any(|a| a == "--json");
            let articles = app.storage.load_articles();
            if is_json {
                println!("{}", serde_json::to_string(&articles).unwrap_or_default());
            } else {
                println!("=== QuickNews Haber Akisi ({} haber) ===", articles.len());
                for (i, art) in articles.iter().enumerate().take(30) {
                    let status = if art.is_read { "[Okundu] " } else { "[YENI]   " };
                    println!(
                        "{:2}. {} [{}] {} ({} dk okuma)",
                        i + 1,
                        status,
                        art.source_name,
                        art.title,
                        art.reading_time_mins
                    );
                }
            }
        }
        "read" => {
            if args.len() < 3 {
                eprintln!("Kullanim: quicknews-engine read <URL> [--json]");
                std::process::exit(1);
            }
            let url = &args[2];
            let is_json = args.iter().any(|a| a == "--json");

            match app.read_clean_article(url).await {
                Ok(clean) => {
                    if is_json {
                        println!("{}", serde_json::to_string(&clean).unwrap_or_default());
                    } else {
                        println!("============================================================");
                        println!("BASLIK: {}", clean.title);
                        if let Some(ref a) = clean.author {
                            println!("YAZAR : {}", a);
                        }
                        if let Some(ref d) = clean.published_date {
                            println!("TARIH : {}", d);
                        }
                        println!("SURE  : Yaklasik {} dakika okuma ({} kelime)", clean.reading_time_mins, clean.word_count);
                        println!("KAYNAK: {}", clean.source_url);
                        println!("============================================================\n");
                        println!("{}", clean.content_text);
                        println!("\n============================================================");
                    }
                }
                Err(e) => {
                    eprintln!("Haber metni cekilemedi: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "summarize" => {
            if args.len() < 3 {
                eprintln!("Kullanim: quicknews-engine summarize <URL> [--json]");
                std::process::exit(1);
            }
            let url = &args[2];
            let is_json = args.iter().any(|a| a == "--json");

            match app.read_clean_article(url).await {
                Ok(clean) => {
                    let summary = AiEngine::summarize_article(&clean.title, &clean.content_text).await;
                    if is_json {
                        println!("{}", serde_json::to_string(&summary).unwrap_or_default());
                    } else {
                        println!("============================================================");
                        println!("YAPAY ZEKA HABER OZETI");
                        println!("NOTR BASLIK: {}", summary.neutral_title);
                        println!("OKUMA SURESI: {} dakika", summary.reading_time_mins);
                        println!("------------------------------------------------------------");
                        println!("3 MADDEDE ONEMLI NOKTALAR:");
                        for (i, p) in summary.key_points.iter().enumerate() {
                            println!(" {}. {}", i + 1, p);
                        }
                        println!("============================================================");
                    }
                }
                Err(e) => {
                    eprintln!("Haber ozeti olusturulamadi: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "sources" => {
            let is_json = args.iter().any(|a| a == "--json");
            let sources = app.storage.load_sources().unwrap_or_default();
            if is_json {
                println!("{}", serde_json::to_string(&sources).unwrap_or_default());
            } else {
                println!("=== Ekli Haber Kaynaklari ({} kaynak) ===", sources.len());
                for s in sources {
                    println!("- [{}] {} ({}) -> {}", s.category, s.name, s.domain, s.feed_url);
                }
            }
        }
        "add-prompt" => {
            if args.len() < 3 {
                eprintln!("Kullanim: quicknews-engine add-prompt \"<Dogal dil arama metni>\" [--json]");
                std::process::exit(1);
            }
            let prompt = &args[2];
            let is_json = args.iter().any(|a| a == "--json");

            if !is_json {
                println!("Kaynaklar araniyor ve dogrulaniyor: \"{}\"...", prompt);
            }

            let added = app.discover_and_add_sources(prompt).await;

            if is_json {
                println!("{}", serde_json::to_string(&added).unwrap_or_default());
            } else {
                println!("Basariyla {} yeni kaynak dogrulandi ve eklendi:", added.len());
                for src in &added {
                    println!(
                        "  + {} ({}) -> {}",
                        src.name,
                        src.category,
                        src.suggested_feed.as_deref().unwrap_or("feed yok")
                    );
                }
            }
        }
        "add-source" => {
            if args.len() < 6 {
                eprintln!("Kullanim: quicknews-engine add-source <Isim> <Domain> <FeedURL> <Kategori>");
                std::process::exit(1);
            }
            match app.storage.add_source(&args[2], &args[3], &args[4], &args[5]) {
                Ok(true) => println!("Kaynak basariyla eklendi."),
                Ok(false) => println!("Bu kaynak zaten mevcut."),
                Err(e) => eprintln!("Kaynak ekleme hatasi: {}", e),
            }
        }
        "remove-source" => {
            if args.len() < 3 {
                eprintln!("Kullanim: quicknews-engine remove-source <ID veya Domain>");
                std::process::exit(1);
            }
            match app.storage.remove_source(&args[2]) {
                Ok(true) => println!("Kaynak silindi."),
                Ok(false) => println!("Kaynak bulunamadi."),
                Err(e) => eprintln!("Silme hatasi: {}", e),
            }
        }
        "mark-read" => {
            if args.len() < 3 {
                eprintln!("Kullanim: quicknews-engine mark-read <HaberID>");
                std::process::exit(1);
            }
            let _ = app.storage.mark_article_read(&args[2]);
            println!("OK");
        }
        "save" => {
            if args.len() < 3 {
                eprintln!("Kullanim: quicknews-engine save <HaberID veya Link>");
                std::process::exit(1);
            }
            let target = &args[2];
            let articles = app.storage.load_articles();
            if let Some(item) = articles.iter().find(|a| a.id == *target || a.link == *target) {
                let _ = app.storage.save_article_bookmark(item);
                println!("OK");
            } else {
                println!("Haber bulunamadi");
            }
        }
        "unsave" => {
            if args.len() < 3 {
                eprintln!("Kullanim: quicknews-engine unsave <HaberID veya Link>");
                std::process::exit(1);
            }
            let _ = app.storage.remove_saved_article(&args[2]);
            println!("OK");
        }
        "toggle-save" => {
            if args.len() < 3 {
                eprintln!("Kullanim: quicknews-engine toggle-save <HaberID veya Link>");
                std::process::exit(1);
            }
            let (is_saved, ok) = app.toggle_save_article(&args[2]);
            let res = serde_json::json!({
                "saved": is_saved,
                "success": ok
            });
            println!("{}", res);
        }
        "saved" => {
            let is_json = args.iter().any(|a| a == "--json");
            let saved = app.load_saved_articles();
            if is_json {
                println!("{}", serde_json::to_string(&saved).unwrap_or_default());
            } else {
                println!("=== Kaydedilen Haberler ({} haber) ===", saved.len());
                for (i, art) in saved.iter().enumerate() {
                    println!("{:2}. [{}] {} ({})", i + 1, art.source_name, art.title, art.link);
                }
            }
        }
        "export" => {
            if args.len() < 3 {
                eprintln!("Kullanim: quicknews-engine export <URL> [--out <DosyaYolu>]");
                std::process::exit(1);
            }
            let url = &args[2];
            let out_idx = args.iter().position(|a| a == "--out").map(|i| i + 1);
            let target_path = if let Some(idx) = out_idx {
                if idx < args.len() {
                    Some(std::path::Path::new(&args[idx]))
                } else {
                    None
                }
            } else {
                None
            };
            match app.export_article_to_markdown(url, target_path).await {
                Ok(path) => println!("Haber basariyla disari aktarildi: {}", path.display()),
                Err(e) => {
                    eprintln!("Disa aktarma hatasi: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "status" => {
            let sources = app.storage.load_sources().unwrap_or_default();
            let articles = app.storage.load_articles();
            let unread_count = articles.iter().filter(|a| !a.is_read).count();

            let status_obj = serde_json::json!({
                "app": "QuickNews",
                "sources_count": sources.len(),
                "articles_count": articles.len(),
                "unread_count": unread_count,
            });
            println!("{}", status_obj);
        }
        "version" | "--version" | "-v" => {
            println!("QuickNews Engine v0.1.0 (Omarchy Linux)");
        }
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("QuickNews Engine - Guvenli, Reklamsiz ve Yapay Zeka Destekli Haber Motoru");
    println!("Kullanim: quicknews-engine <komut> [parametreler]\n");
    println!("Komutlar:");
    println!("  sync                  Tum kaynaklardan son haberleri ceker");
    println!("  list [--json]         Onbellekteki haberleri listeler");
    println!("  read <URL> [--json]   Haberin saf, reklamsiz ve resimsiz metnini okur");
    println!("  summarize <URL>       Yapay zeka ile 3 maddelik hap ozet ve notr baslik cikarir");
    println!("  sources [--json]      Kayitli haber kaynaklarini listeler");
    println!("  add-prompt \"<metin>\"  Dogal dille otomatik kaynak kesfeder ve ekler");
    println!("  add-source            Manuel haber kaynagi ekler");
    println!("  remove-source <id>    Haber kaynagini kaldirir");
    println!("  mark-read <id>        Haberi okundu olarak isaretler");
    println!("  status                Uygulama durumunu JSON formatinda dondurur");
}
