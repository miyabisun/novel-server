use std::collections::HashSet;

pub fn clean(html: &str) -> String {
    let mut allowed = HashSet::new();
    for tag in &[
        "p", "br", "hr", "div", "span", "h1", "h2", "h3", "h4", "h5", "h6", "ruby", "rt", "rp",
        "rb", "em", "strong", "b", "i", "u", "s", "sub", "sup",
    ] {
        allowed.insert(*tag);
    }

    ammonia::Builder::default()
        .tags(allowed)
        .clean_content_tags(
            ["script", "style", "title", "noscript", "template"]
                .iter()
                .copied()
                .collect(),
        )
        .generic_attributes(HashSet::new())
        .tag_attributes(std::collections::HashMap::new())
        .strip_comments(true)
        .clean(html)
        .to_string()
}
