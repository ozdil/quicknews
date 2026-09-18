pragma Singleton
import QtQuick
import Quickshell

QtObject {
    id: root

    // Default language is US English
    property string currentLanguage: "en"

    Component.onCompleted: {
        detectSystemLanguage();
    }

    function detectSystemLanguage() {
        var lang = "";
        try {
            var envLang = Quickshell.env("LC_ALL") || Quickshell.env("LC_MESSAGES") || Quickshell.env("LANG") || "";
            lang = String(envLang).trim().toLowerCase();
        } catch(e) {
            lang = "";
        }

        if (lang.indexOf("tr") === 0 || lang.indexOf("turkish") !== -1) {
            root.currentLanguage = "tr";
        } else {
            // Default to English (US)
            root.currentLanguage = "en";
        }
    }

    function setLanguage(lang) {
        if (lang === "tr" || lang === "en") {
            root.currentLanguage = lang;
        }
    }

    // Translation dictionary: English (US) is default, Turkish supported
    readonly property var translations: ({
        // App Header & Navigation
        "app_title": { "en": "QuickNews", "tr": "QuickNews" },
        "pure_text_reader": { "en": "PURE TEXT READER", "tr": "SAF METIN OKUYUCU" },
        "ai_summary": { "en": "AI Summary", "tr": "Yapay Zeka Ozeti" },
        "summarizing": { "en": "Summarizing...", "tr": "Ozetleniyor..." },
        "mark_read": { "en": "Mark as Read", "tr": "Okundu Yap" },
        "mark_read_btn": { "en": "Mark as Read", "tr": "Okundu Olarak Isaretle" },
        "article_completed": { "en": "You completed this article", "tr": "Haberi tamamladiniz" },
        "article_completed_sub": { "en": "Mark as read and dismiss from list", "tr": "Okundu olarak isaretleyip listeden kaldirin" },
        "empty_reader_title": { "en": "Select an article from the list to read", "tr": "Okumak istediginiz haberi listeden secin" },
        "empty_reader_sub": { "en": "Zero ads • Zero images • Pure distraction-free text", "tr": "Sifir reklam • Sifir resim • Tamamen saf metin" },
        "extracting_content": { "en": "Extracting clean distraction-free text...", "tr": "Haberin reklamsiz ve saf metni cikariliyor..." },
        "ai_analysis": { "en": "AI NEWS ANALYSIS", "tr": "YAPAY ZEKA HABER ANALIZI" },
        "neutral_headline": { "en": "Neutral Headline: ", "tr": "Notr Baslik: " },
        "min_read": { "en": "min read", "tr": "dk okuma" },

        // Sidebar
        "ai_add_source": { "en": "Add Source with AI", "tr": "AI ile Kaynak Ekle" },
        "categories": { "en": "CATEGORIES", "tr": "KATEGORILER" },
        "sources": { "en": "SOURCES", "tr": "KAYNAKLAR" },
        "new_count": { "en": "new", "tr": "yeni" },
        "remove_source": { "en": "Remove source", "tr": "Kaynaktan cikar" },
        "footer_tagline": { "en": "Omarchy Linux • Distraction-free & Secure", "tr": "Omarchy Linux • Resimsiz ve Guvenli" },

        // Categories Names
        "cat_all": { "en": "All", "tr": "Tümü" },
        "cat_agenda": { "en": "News", "tr": "Gündem" },
        "cat_politics": { "en": "Politics", "tr": "Siyaset" },
        "cat_local": { "en": "Local", "tr": "Yerel" },
        "cat_tech": { "en": "Technology", "tr": "Teknoloji" },
        "cat_linux": { "en": "Linux", "tr": "Linux" },
        "cat_gaming": { "en": "Gaming", "tr": "Oyun" },
        "cat_hardware": { "en": "Hardware", "tr": "Donanım" },
        "cat_science": { "en": "Science", "tr": "Bilim" },
        "cat_cybersecurity": { "en": "Cybersecurity", "tr": "Siber Güvenlik" },
        "cat_startups": { "en": "Startups", "tr": "Girişimcilik" },

        // Headlines List
        "headlines": { "en": "HEADLINES", "tr": "BASLIKLAR" },
        "search_placeholder": { "en": "Filter headlines... (/)", "tr": "Basliklarda ara... (/)" },
        "tab_unread": { "en": "Unread", "tr": "Okunmamis" },
        "tab_all": { "en": "All Articles", "tr": "Tum Haberler" },
        "tab_saved": { "en": "Saved", "tr": "Kaydedilenler" },
        "no_articles": { "en": "No articles in this feed", "tr": "Bu akista haber bulunamadi" },
        "sync_now": { "en": "Sync Feeds (R)", "tr": "Akislari Guncelle (R)" },
        "just_now": { "en": "just now", "tr": "az once" },
        "hours_ago": { "en": "hours ago", "tr": "saat once" },
        "yesterday": { "en": "yesterday", "tr": "dun" },

        // Add Source Modal
        "modal_title": { "en": "Add News Source with AI", "tr": "Yapay Zeka ile Haber Kaynagi Ekle" },
        "modal_sub": { "en": "Describe topics, cities or sites you want to follow in plain language", "tr": "Takip etmek istediginiz konulari, sehirleri veya siteleri dogal dille yazin" },
        "modal_input_placeholder": { "en": "e.g. Add local news for Austin and Texas tech blogs...", "tr": "Orn: Izmir yerel haberleri ve yapay zeka gelismelerini ekle..." },
        "modal_searching": { "en": "Analyzing sources and discovering RSS feeds...", "tr": "Yapay zeka kaynaklari analiz ediyor ve RSS akislarini dogruluyor..." },
        "modal_cancel": { "en": "Cancel (Esc)", "tr": "Vazgec (Esc)" },
        "modal_submit": { "en": "Search and Add", "tr": "Ara ve Ekle" },
        "modal_success": { "en": "Successfully added new sources!", "tr": "Yeni kaynaklar basariyla eklendi!" },
        "modal_controlled": { "en": "Sources checked.", "tr": "Kaynaklar kontrol edildi." }
    })

    function t(key) {
        if (!key) return "";
        var item = translations[key];
        if (!item) return key;
        var lang = root.currentLanguage || "en";
        return item[lang] || item["en"] || key;
    }

    function formatDate(rawDate) {
        if (!rawDate || rawDate.length === 0) return "";
        try {
            var d = new Date(rawDate);
            if (isNaN(d.getTime())) {
                return rawDate;
            }
            var hours = ("0" + d.getHours()).slice(-2);
            var minutes = ("0" + d.getMinutes()).slice(-2);
            var day = d.getDate();
            var year = d.getFullYear();

            if (root.currentLanguage === "tr") {
                var monthsTr = [
                    "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran",
                    "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"
                ];
                return day + " " + monthsTr[d.getMonth()] + " " + year + ", " + hours + ":" + minutes;
            } else {
                // English (US) default
                var monthsEn = [
                    "January", "February", "March", "April", "May", "June",
                    "July", "August", "September", "October", "November", "December"
                ];
                return monthsEn[d.getMonth()] + " " + day + ", " + year + ", " + hours + ":" + minutes;
            }
        } catch (e) {
            return rawDate;
        }
    }
}
