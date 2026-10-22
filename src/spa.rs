use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

static CACHED_HTML: Mutex<Option<(String, SystemTime)>> = Mutex::new(None);

pub fn get_index_html(base_path: &str) -> Option<String> {
    let index_path = Path::new("client/build/index.html");

    // In production, return cached version
    let is_prod = std::env::var("NODE_ENV").is_ok_and(|v| v == "production");

    let mut cached = CACHED_HTML.lock().expect("lock holders never panic");

    let mtime = fs::metadata(index_path).ok()?.modified().ok()?;

    if let Some((ref html, ref cached_mtime)) = *cached {
        if is_prod || mtime == *cached_mtime {
            return Some(html.clone());
        }
    }

    let raw = fs::read_to_string(index_path).ok()?;
    let html = raw
        .replace(
            "<head>",
            &format!("<head>\n\t\t<base href=\"{base_path}/\">"),
        )
        .replace(
            "window.__BASE_PATH__ = \"\"",
            &format!(
                "window.__BASE_PATH__ = {}",
                serde_json::to_string(base_path).expect("a string always serializes")
            ),
        );

    *cached = Some((html.clone(), mtime));
    Some(html)
}
