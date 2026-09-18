/// Ad-blocking, tracking protection, and clean content filtering rules.
pub struct AdBlocker;

impl AdBlocker {
    /// Checks if an HTML tag name should be unconditionally eliminated.
    pub fn is_forbidden_tag(tag: &str) -> bool {
        matches!(
            tag.to_lowercase().as_str(),
            "script"
                | "style"
                | "noscript"
                | "iframe"
                | "frame"
                | "frameset"
                | "object"
                | "embed"
                | "applet"
                | "form"
                | "input"
                | "button"
                | "select"
                | "textarea"
                | "header"
                | "footer"
                | "nav"
                | "aside"
                | "svg"
                | "canvas"
                | "audio"
                | "video"
                | "picture"
                | "img" // Strictly image-free / text-first policy
                | "source"
                | "dialog"
                | "menu"
        )
    }

    /// Checks if a class or id attribute indicates an advertisement, tracker, cookie notice, or widget.
    pub fn is_ad_or_clutter_attribute(value: &str) -> bool {
        let v = value.to_lowercase();
        let patterns = [
            "advert",
            "sponsor",
            "banner",
            "cookie",
            "consent",
            "gdpr",
            "modal",
            "popup",
            "newsletter",
            "subscribe",
            "outbrain",
            "taboola",
            "ad-box",
            "ad_box",
            "ad-container",
            "ad-wrapper",
            "ad-slot",
            "dfp-",
            "google-ad",
            "adsense",
            "share-buttons",
            "social-share",
            "social-media",
            "sharing",
            "related-articles",
            "recommended-news",
            "tab-nav",
            "breadcrumbs",
            "comments",
            "disqus",
            "paywall",
            "more-contents",
            "content-end",
            "support-bottom",
            "donation",
            "sharebar",
            "author-box",
            "print-tool",
            "content-support",
            "related",
            "inloop",
            "check-this-out",
            "carousel",
            "listing",
            "post-navigation",
            "recommended",
            "similar",
            "widget-post",
            "entry-related",
            "popular-post",
        ];

        for p in &patterns {
            if v.contains(p) {
                return true;
            }
        }
        false
    }

    /// Strips tracking query parameters from URLs (e.g. utm_*, fbclid).
    pub fn clean_url_tracking(raw_url: &str) -> String {
        if let Ok(mut parsed) = url::Url::parse(raw_url) {
            let filtered: Vec<(String, String)> = parsed
                .query_pairs()
                .filter(|(k, _)| {
                    let key = k.to_lowercase();
                    !key.starts_with("utm_")
                        && key != "fbclid"
                        && key != "gclid"
                        && key != "yclid"
                        && key != "msclkid"
                        && key != "ref"
                        && key != "source"
                        && key != "spm"
                })
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            if filtered.is_empty() {
                parsed.set_query(None);
            } else {
                let mut new_query = String::new();
                for (i, (k, v)) in filtered.iter().enumerate() {
                    if i > 0 {
                        new_query.push('&');
                    }
                    new_query.push_str(k);
                    new_query.push('=');
                    new_query.push_str(v);
                }
                parsed.set_query(Some(&new_query));
            }
            parsed.to_string()
        } else {
            raw_url.to_string()
        }
    }
}
