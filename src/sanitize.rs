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
