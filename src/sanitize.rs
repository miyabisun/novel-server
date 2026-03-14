use ammonia::Builder;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static SANITIZER: LazyLock<Builder<'static>> = LazyLock::new(|| {
    let allowed: HashSet<&str> = [
        "p", "br", "hr", "div", "span", "h1", "h2", "h3", "h4", "h5", "h6", "ruby", "rt", "rp",
        "rb", "em", "strong", "b", "i", "u", "s", "sub", "sup",
    ]
    .into_iter()
    .collect();

    let clean_content: HashSet<&str> = ["script", "style", "title", "noscript", "template"]
        .into_iter()
        .collect();

    // Strip all attributes unconditionally. Scraped HTML may contain event handler
    // attributes (onclick, onerror, etc.), and an allow/deny-list approach for
    // individual attributes risks omissions.
    let mut builder = Builder::default();
    builder
        .tags(allowed)
        .clean_content_tags(clean_content)
        .generic_attributes(HashSet::new())
        .tag_attributes(HashMap::new())
        .strip_comments(true);
    builder
});

pub fn clean(html: &str) -> String {
    SANITIZER.clean(html).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_allowed_tags() {
        let html = "<p>text</p><br><hr><div>d</div><span>s</span>";
        let out = clean(html);
        assert!(out.contains("<p>"));
        assert!(out.contains("<br>"));
        assert!(out.contains("<hr>"));
        assert!(out.contains("<div>"));
        assert!(out.contains("<span>"));
    }

    #[test]
    fn preserves_heading_tags() {
        for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
            let html = format!("<{tag}>heading</{tag}>");
            assert!(clean(&html).contains(&format!("<{tag}>")), "missing {tag}");
        }
    }

    #[test]
    fn preserves_ruby_tags() {
        let html = "<ruby>漢<rt>かん</rt><rp>(</rp><rb>字</rb><rp>)</rp></ruby>";
        let out = clean(html);
        assert!(out.contains("<ruby>"));
        assert!(out.contains("<rt>"));
        assert!(out.contains("<rp>"));
        assert!(out.contains("<rb>"));
    }

    #[test]
    fn preserves_inline_formatting_tags() {
        let html = "<em>e</em><strong>s</strong><b>b</b><i>i</i><u>u</u><s>s</s><sub>1</sub><sup>2</sup>";
        let out = clean(html);
        for tag in ["em", "strong", "b", "i", "u", "s", "sub", "sup"] {
            assert!(out.contains(&format!("<{tag}>")), "missing {tag}");
        }
    }

    #[test]
    fn strips_script_tag_and_content() {
        let html = "<p>safe</p><script>alert('xss')</script>";
        let out = clean(html);
        assert!(!out.contains("script"));
        assert!(!out.contains("alert"));
        assert!(out.contains("safe"));
    }

    #[test]
    fn strips_style_tag_and_content() {
        let html = "<p>text</p><style>body{display:none}</style>";
        let out = clean(html);
        assert!(!out.contains("style"));
        assert!(!out.contains("display"));
    }

    #[test]
    fn strips_img_onerror_xss() {
        let html = r#"<img src=x onerror="alert('xss')">"#;
        let out = clean(html);
        assert!(!out.contains("onerror"));
        assert!(!out.contains("alert"));
    }

    #[test]
    fn strips_all_attributes_from_allowed_tags() {
        let html = r#"<p class="foo" style="color:red" id="bar" onclick="evil()">text</p>"#;
        let out = clean(html);
        assert_eq!(out, "<p>text</p>");
    }

    #[test]
    fn strips_event_handler_attributes() {
        let html = r#"<div onmouseover="steal()">hover</div>"#;
        let out = clean(html);
        assert!(!out.contains("onmouseover"));
        assert!(out.contains("hover"));
    }

    #[test]
    fn keeps_text_from_non_allowed_tags() {
        let html = "<a href='http://evil.com'>link text</a>";
        let out = clean(html);
        assert!(!out.contains("<a"));
        assert!(out.contains("link text"));
    }

    #[test]
    fn strips_html_comments() {
        let html = "<p>text</p><!-- secret comment -->";
        let out = clean(html);
        assert!(!out.contains("<!--"));
        assert!(!out.contains("secret"));
    }

    #[test]
    fn handles_empty_input() {
        assert_eq!(clean(""), "");
    }

    #[test]
    fn handles_plain_text() {
        assert_eq!(clean("hello world"), "hello world");
    }
}
