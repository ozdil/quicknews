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
                    let tags_str = if art.tags.is_empty() { String::new() } else { format!(" [{}]", art.tags.join("/")) };
                    println!(
                        "{:2}. {} [{}] {}{} ({} dk okuma)",
                        i + 1,
                        status,
                        art.source_name,
                        art.title,
                        tags_str,
                        art.reading_time_mins
                    );
                }
            }
        }
        "read" => {
            let url = match find_positional_arg(&args, 2) {
                Some(u) => u,
                None => {
                    eprintln!("Kullanim: quicknews-engine read [--] <URL> [--json]");
                    std::process::exit(1);
                }
            };
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
                        if !clean.tags.is_empty() {
                            println!("ETIKET: {}", clean.tags.join(" | "));
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
            let url = match find_positional_arg(&args, 2) {
                Some(u) => u,
                None => {
                    eprintln!("Kullanim: quicknews-engine summarize [--] <URL> [--json]");
                    std::process::exit(1);
                }
            };
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
            let prompt = match find_positional_arg(&args, 2) {
                Some(p) => p,
                None => {
                    eprintln!("Kullanim: quicknews-engine add-prompt [--] \"<Dogal dil arama metni>\" [--json]");
                    std::process::exit(1);
                }
            };
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
            let is_json = args.iter().any(|a| a == "--json");
            let filtered: Vec<&str> = args.iter().skip(2).filter(|a| *a != "--" && *a != "--json").map(|s| s.as_str()).collect();
            if filtered.len() < 3 {
                eprintln!("Kullanim: quicknews-engine add-source <Isim> <Domain veya FeedURL> <Kategori> [FeedURL]");
                std::process::exit(1);
            }
            let name = filtered[0];
            let domain_or_url = filtered[1];
            let category = filtered[2];
            let feed_url = if filtered.len() >= 4 { filtered[3] } else { domain_or_url };

            // Extract clean domain
            let domain = if domain_or_url.starts_with("http://") || domain_or_url.starts_with("https://") {
                url::Url::parse(domain_or_url)
                    .ok()
                    .and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string()))
                    .unwrap_or_else(|| domain_or_url.to_string())
            } else {
                domain_or_url.trim_start_matches("www.").to_string()
            };

            match app.add_source_verified(name, &domain, feed_url, category).await {
                Ok(true) => {
                    if is_json {
                        println!("{}", serde_json::json!({ "success": true, "message": "Kaynak basariyla dogrulandi ve eklendi." }));
                    } else {
                        println!("Kaynak basariyla dogrulandi ve eklendi.");
                    }
                }
                Ok(false) => {
                    if is_json {
                        println!("{}", serde_json::json!({ "success": false, "message": "Bu kaynak zaten mevcut." }));
                    } else {
                        println!("Bu kaynak zaten mevcut.");
                    }
                }
                Err(e) => {
                    if is_json {
                        println!("{}", serde_json::json!({ "success": false, "error": e.to_string() }));
                    } else {
                        eprintln!("Kaynak ekleme hatasi: {}", e);
                    }
                    std::process::exit(1);
                }
            }
        }
        "remove-source" => {
            let target = match find_positional_arg(&args, 2) {
                Some(t) => t,
                None => {
                    eprintln!("Kullanim: quicknews-engine remove-source [--] <ID veya Domain>");
                    std::process::exit(1);
                }
            };
            match app.storage.remove_source(target) {
                Ok(true) => println!("Kaynak silindi."),
                Ok(false) => println!("Kaynak bulunamadi."),
                Err(e) => eprintln!("Silme hatasi: {}", e),
            }
        }
        "mark-read" => {
            let target = match find_positional_arg(&args, 2) {
                Some(t) => t,
                None => {
                    eprintln!("Kullanim: quicknews-engine mark-read [--] <HaberID veya Link>");
                    std::process::exit(1);
                }
            };
            let _ = app.storage.mark_article_read(target);
            println!("OK");
        }
        "mark-unread" => {
            let target = match find_positional_arg(&args, 2) {
                Some(t) => t,
                None => {
                    eprintln!("Kullanim: quicknews-engine mark-unread [--] <HaberID veya Link>");
                    std::process::exit(1);
                }
            };
            let _ = app.storage.set_article_read_state(target, false);
            println!("OK");
        }
        "toggle-read" => {
            let target = match find_positional_arg(&args, 2) {
                Some(t) => t,
                None => {
                    eprintln!("Kullanim: quicknews-engine toggle-read [--] <HaberID veya Link>");
                    std::process::exit(1);
                }
            };
            let (is_read, ok) = app.toggle_read_article(target);
            let res = serde_json::json!({
                "id": target,
                "is_read": is_read,
                "success": ok
            });
            println!("{}", res);
        }
        "save" => {
            let target = match find_positional_arg(&args, 2) {
                Some(t) => t,
                None => {
                    eprintln!("Kullanim: quicknews-engine save [--] <HaberID veya Link>");
                    std::process::exit(1);
                }
            };
            let articles = app.storage.load_articles();
            if let Some(item) = articles.iter().find(|a| a.id == target || a.link == target) {
                let _ = app.storage.save_article_bookmark(item);
                println!("OK");
            } else {
                println!("Haber bulunamadi");
            }
        }
        "unsave" => {
            let target = match find_positional_arg(&args, 2) {
                Some(t) => t,
                None => {
                    eprintln!("Kullanim: quicknews-engine unsave [--] <HaberID veya Link>");
                    std::process::exit(1);
                }
            };
            let _ = app.storage.remove_saved_article(target);
            println!("OK");
        }
        "toggle-save" => {
            let target = match find_positional_arg(&args, 2) {
                Some(t) => t,
                None => {
                    eprintln!("Kullanim: quicknews-engine toggle-save [--] <HaberID veya Link>");
                    std::process::exit(1);
                }
            };
            let (is_saved, ok) = app.toggle_save_article(target);
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
            let url = match find_positional_arg(&args, 2) {
                Some(u) => u,
                None => {
                    eprintln!("Kullanim: quicknews-engine export [--] <URL> [--out <DosyaYolu>]");
                    std::process::exit(1);
                }
            };
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
            let is_json = args.iter().any(|a| a == "--json");
            match app.export_article_to_markdown(url, target_path).await {
                Ok(path) => {
                    if is_json {
                        println!("{}", serde_json::json!({
                            "success": true,
                            "path": path.display().to_string()
                        }));
                    } else {
                        println!("Haber basariyla disari aktarildi: {}", path.display());
                    }
                }
                Err(e) => {
                    if is_json {
                        println!("{}", serde_json::json!({
                            "success": false,
                            "error": e.to_string()
                        }));
                    } else {
                        eprintln!("Disa aktarma hatasi: {}", e);
                    }
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
            println!("QuickNews Engine v0.2.0 (Omarchy Linux)");
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
    println!("  mark-unread <id>      Haberi okunmadi olarak isaretler");
    println!("  toggle-read <id>      Haberin okundu/okunmadi durumunu degistirir");
    println!("  status                Uygulama durumunu JSON formatinda dondurur");
}

fn find_positional_arg(args: &[String], skip: usize) -> Option<&str> {
    let mut i = skip;
    while i < args.len() {
        let a = &args[i];
        if a == "--" {
            if i + 1 < args.len() {
                return Some(&args[i + 1]);
            }
            return None;
        }
        if a == "--out" {
            i += 2;
            continue;
        }
        if a == "--json" {
            i += 1;
            continue;
        }
        return Some(a.as_str());
    }
    None
}
