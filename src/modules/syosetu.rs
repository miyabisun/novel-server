use crate::error::AppError;
use scraper::{Html, Selector};
use serde_json::{json, Map, Value};

/// Normalize a single API response item: ncode->id, title->trim, general_all_no->page
pub fn map_item(obj: &Value) -> Value {
    let map = match obj.as_object() {
        Some(m) => m,
        None => return obj.clone(),
    };
    let mut acc = Map::new();
    for (key, val) in map {
        match key.as_str() {
            "ncode" => {
                let id = val
                    .as_str()
                    .map(|s| Value::String(s.to_lowercase()))
                    .unwrap_or_else(|| val.clone());
                acc.insert("id".to_string(), id);
            }
            "title" => {
                let title = val
                    .as_str()
                    .map(|s| Value::String(s.trim().to_string()))
                    .unwrap_or_else(|| val.clone());
                acc.insert("title".to_string(), title);
            }
            "general_all_no" => {
                acc.insert("page".to_string(), val.clone());
            }
            _ => {
                acc.insert(key.clone(), val.clone());
            }
        }
    }
    Value::Object(acc)
}

/// Build a pages array for a given novel
pub fn build_pages(type_str: &str, id: &str, count: u64) -> Value {
    let pages: Vec<Value> = (1..=count)
        .map(|i| {
            json!({
                "type": type_str,
                "id": id,
                "num": i,
                "page_id": i.to_string(),
            })
        })
        .collect();
    Value::Array(pages)
}

/// Fetch from syosetu API, parse JSON, skip first element (metadata), map items
pub async fn fetch_api(
    client: &reqwest::Client,
    api_url: &str,
    params: &[(&str, String)],
    headers: Option<reqwest::header::HeaderMap>,
) -> Result<Vec<Value>, AppError> {
    let mut all_params: Vec<(&str, String)> = vec![("out", "json".to_string())];
    all_params.extend_from_slice(params);

    let mut req = client.get(api_url);
    req = req.query(&all_params);
    if let Some(hdrs) = headers {
        for (name, val) in hdrs.iter() {
            req = req.header(name, val);
        }
    }

    let res = req.send().await?;
    if !res.status().is_success() {
        return Err(AppError::Upstream(format!("API error: {}", res.status())));
    }
    let json: Vec<Value> = res.json().await?;
    Ok(process_api_response(json))
}

/// Process raw syosetu API JSON response:
/// skip first element (metadata/count), filter out non-object entries, map items
pub fn process_api_response(json: Vec<Value>) -> Vec<Value> {
    json.into_iter()
        .skip(1)
        .filter(|v| v.is_object())
        .map(|v| map_item(&v))
        .collect()
}

/// Parse a TOC HTML page from syosetu
pub fn parse_toc(html: &str) -> TocResult {
    let doc = Html::parse_document(html);

    // Title: .p-novel__title or <title>
    let title_sel = Selector::parse(".p-novel__title").unwrap();
    let title_fallback_sel = Selector::parse("title").unwrap();
    let title = doc
        .select(&title_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            doc.select(&title_fallback_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
        })
        .unwrap_or_default();

    // Episodes: .p-eplist__sublist
    let ep_sel = Selector::parse(".p-eplist__sublist").unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let mut episodes = Vec::new();
    let mut num = 0u64;
    for el in doc.select(&ep_sel) {
        num += 1;
        let ep_title = el
            .select(&a_sel)
            .next()
            .map(|a| a.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        episodes.push(TocEpisode {
            num,
            title: ep_title,
        });
    }

    // Last page: find <a> with text "最後へ" and extract ?p=N
    let a_all_sel = Selector::parse("a").unwrap();
    let mut last_page = 1u64;
    for el in doc.select(&a_all_sel) {
        let text: String = el.text().collect();
        if text.trim() == "最後へ" {
            if let Some(href) = el.value().attr("href") {
                if let Some(cap) = href.find("p=") {
                    let rest = &href[cap + 2..];
                    let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if let Ok(n) = num_str.parse::<u64>() {
                        last_page = n;
                    }
                }
            }
        }
    }

    TocResult {
        title,
        episodes,
        last_page,
    }
}

/// Parse a novel page: select elements by CSS selector, join with <hr>
pub fn parse_page(html: &str, selector: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse(selector).ok()?;
    let parts: Vec<String> = doc
        .select(&sel)
        .map(|el| el.inner_html())
        .filter(|h| !h.trim().is_empty())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("<hr>"))
    }
}

pub struct TocResult {
    pub title: String,
    pub episodes: Vec<TocEpisode>,
    pub last_page: u64,
}

pub struct TocEpisode {
    pub num: u64,
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── process_api_response: null entry filtering (regression for #1) ──

    #[test]
    fn process_api_response_filters_null_entries() {
        // Simulates a corrupted API response where items are null
        // (the bug that crashed the frontend with "can't access property 'id', l(...) is null")
        let raw = vec![
            json!({"allcount": 3}),
            Value::Null,
            Value::Null,
            Value::Null,
        ];
        let result = process_api_response(raw);
        assert!(result.is_empty(), "null entries must be filtered out");
    }

    #[test]
    fn process_api_response_filters_mixed_null_and_valid() {
        let raw = vec![
            json!({"allcount": 3}),
            json!({"ncode": "N1234AB", "title": "Valid Novel", "general_all_no": 10}),
            Value::Null,
            json!({"ncode": "N5678CD", "title": "Another", "general_all_no": 5}),
        ];
        let result = process_api_response(raw);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["id"], "n1234ab");
        assert_eq!(result[1]["id"], "n5678cd");
    }

    #[test]
    fn process_api_response_filters_non_object_primitives() {
        let raw = vec![
            json!({"allcount": 1}),
            json!(42),
            json!("string"),
            json!(true),
            json!({"ncode": "N0001AA", "title": "OK", "general_all_no": 1}),
        ];
        let result = process_api_response(raw);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["id"], "n0001aa");
    }

    #[test]
    fn process_api_response_skips_metadata_first_element() {
        let raw = vec![
            json!({"allcount": 1}),
            json!({"ncode": "N0001AA", "title": "First", "general_all_no": 1}),
        ];
        let result = process_api_response(raw);
        assert_eq!(result.len(), 1);
        // The metadata element (allcount) must not appear
        assert_eq!(result[0]["id"], "n0001aa");
    }

    #[test]
    fn process_api_response_normal_response() {
        let raw = vec![
            json!({"allcount": 2}),
            json!({"ncode": "N1111AA", "title": "  Novel One  ", "general_all_no": 42, "novelupdated_at": "2026-01-01"}),
            json!({"ncode": "N2222BB", "title": "Novel Two", "general_all_no": 7}),
        ];
        let result = process_api_response(raw);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["id"], "n1111aa");
        assert_eq!(result[0]["title"], "Novel One");
        assert_eq!(result[0]["page"], 42);
        assert_eq!(result[0]["novelupdated_at"], "2026-01-01");
        assert_eq!(result[1]["id"], "n2222bb");
        assert_eq!(result[1]["page"], 7);
    }
}
