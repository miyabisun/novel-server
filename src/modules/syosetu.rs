use crate::error::AppError;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
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
    for (i, el) in doc.select(&ep_sel).enumerate() {
        let num = (i as u64) + 1;
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

// ── Site-specific configuration and functions ──

pub struct SyosetuSite {
    pub api_url: &'static str,
    pub base_url: &'static str,
    pub type_str: &'static str,
    pub genre_param: &'static str,
    pub ranking_genres: &'static [(&'static str, u32)],
    pub over18: bool,
}

pub static NAROU: SyosetuSite = SyosetuSite {
    api_url: "https://api.syosetu.com/novelapi/api/",
    base_url: "https://ncode.syosetu.com",
    type_str: "narou",
    genre_param: "genre",
    ranking_genres: &[
        ("異世界 [恋愛]", 101),
        ("現実世界 [恋愛]", 102),
        ("ハイファンタジー", 201),
        ("ローファンタジー", 202),
        ("アクション", 306),
    ],
    over18: false,
};

pub static NOCTURNE: SyosetuSite = SyosetuSite {
    api_url: "https://api.syosetu.com/novel18api/api/",
    base_url: "https://novel18.syosetu.com",
    type_str: "nocturne",
    genre_param: "nocgenre",
    ranking_genres: &[("ノクターン", 1)],
    over18: true,
};

/// Output fields for ranking/search (title, writer, ncode, general_all_no, noveltype)
const OF_RANKING: &str = "t-w-n-ga-nt";
/// Output fields for datum/data (ncode, title, general_all_no, story, novelupdated_at)
const OF_DATUM: &str = "n-t-ga-s-nu";
/// Output fields for detail (title, story, general_all_no)
const OF_DETAIL: &str = "t-s-ga";

fn with_headers(site: &SyosetuSite, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    if site.over18 {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_static("over18=yes"));
        req.headers(headers)
    } else {
        req
    }
}

fn to_datum(site: &SyosetuSite, datum: &Value) -> Value {
    let id = datum["id"].as_str().unwrap_or_default();
    let page_count = datum["page"].as_u64().unwrap_or(0);
    let pages = build_pages(site.type_str, id, page_count);
    let mut obj = datum.as_object().cloned().unwrap_or_default();
    obj.remove("page");
    obj.insert("type".to_string(), json!(site.type_str));
    obj.insert("pages".to_string(), pages);
    Value::Object(obj)
}

fn toc_to_json(title: &str, episodes: &[TocEpisode]) -> Value {
    let eps: Vec<Value> = episodes
        .iter()
        .map(|e| json!({"num": e.num, "title": e.title}))
        .collect();
    json!({
        "title": title,
        "episodes": eps,
    })
}

async fn site_api(
    site: &SyosetuSite,
    client: &reqwest::Client,
    params: &[(&str, String)],
) -> Result<Vec<Value>, AppError> {
    fetch_api(client, site.api_url, params, None).await
}

async fn fetch_ranking(
    site: &'static SyosetuSite,
    client: &reqwest::Client,
    genre: u32,
    limit: usize,
    order: &str,
) -> Result<Vec<Value>, AppError> {
    site_api(
        site,
        client,
        &[
            ("of", OF_RANKING.to_string()),
            ("lim", limit.to_string()),
            ("order", order.to_string()),
            (site.genre_param, genre.to_string()),
        ],
    )
    .await
}

async fn fetch_overall_ranking(
    site: &'static SyosetuSite,
    client: &reqwest::Client,
    limit: usize,
    order: &str,
) -> Result<Vec<Value>, AppError> {
    site_api(
        site,
        client,
        &[
            ("of", OF_RANKING.to_string()),
            ("lim", limit.to_string()),
            ("order", order.to_string()),
        ],
    )
    .await
}

pub async fn fetch_ranking_list(
    site: &'static SyosetuSite,
    client: &reqwest::Client,
    limit: usize,
    period: &str,
) -> Result<Value, AppError> {
    let order = match period {
        "daily" => "dailypoint",
        "weekly" => "weeklypoint",
        "monthly" => "monthlypoint",
        "quarter" => "quarterpoint",
        "yearly" => "yearlypoint",
        _ => "dailypoint",
    };

    let mut handles = Vec::new();
    for &(_, genre_id) in site.ranking_genres {
        let client = client.clone();
        let order = order.to_string();
        handles.push(tokio::spawn(async move {
            fetch_ranking(site, &client, genre_id, limit, &order).await
        }));
    }

    // Fetch overall ranking only for sites with multiple genres.
    // For single-genre sites (e.g. nocturne), the genre IS the overall ranking,
    // and fetching without genre filter would include unrelated works.
    let overall_handle = if site.ranking_genres.len() > 1 {
        let overall_client = client.clone();
        let overall_order = order.to_string();
        Some(tokio::spawn(async move {
            fetch_overall_ranking(site, &overall_client, limit, &overall_order).await
        }))
    } else {
        None
    };

    let mut result = serde_json::Map::new();

    for (i, handle) in handles.into_iter().enumerate() {
        let data = handle
            .await
            .map_err(|e| AppError::Internal(e.to_string()))??;
        result.insert(site.ranking_genres[i].0.to_string(), Value::Array(data));
    }

    if let Some(handle) = overall_handle {
        let overall_data = handle
            .await
            .map_err(|e| AppError::Internal(e.to_string()))??;
        result.insert("総合".to_string(), Value::Array(overall_data));
    } else if site.ranking_genres.len() == 1 {
        // Single genre: reuse as "総合"
        let single = result.values().next().cloned().unwrap_or(Value::Array(vec![]));
        result.insert("総合".to_string(), single);
    }
    Ok(Value::Object(result))
}

pub async fn fetch_datum(
    site: &SyosetuSite,
    client: &reqwest::Client,
    id: &str,
) -> Result<Value, AppError> {
    let data = site_api(
        site,
        client,
        &[("of", OF_DATUM.to_string()), ("ncode", id.to_string())],
    )
    .await?;
    data.first()
        .map(|d| to_datum(site, d))
        .ok_or_else(|| AppError::Upstream("Novel not found".to_string()))
}

pub async fn fetch_data(
    site: &SyosetuSite,
    client: &reqwest::Client,
    ids: &[String],
) -> Result<Vec<Value>, AppError> {
    let mut all = Vec::new();
    for chunk in ids.chunks(500) {
        let ncode_str = chunk.join("-");
        let data = site_api(
            site,
            client,
            &[
                ("of", OF_DATUM.to_string()),
                ("ncode", ncode_str),
                ("lim", chunk.len().to_string()),
            ],
        )
        .await?;
        all.extend(data.iter().map(|d| to_datum(site, d)));
    }
    Ok(all)
}

pub async fn fetch_detail(
    site: &SyosetuSite,
    client: &reqwest::Client,
    id: &str,
) -> Result<Value, AppError> {
    let data = site_api(
        site,
        client,
        &[("of", OF_DETAIL.to_string()), ("ncode", id.to_string())],
    )
    .await?;
    let item = data
        .first()
        .ok_or_else(|| AppError::Upstream("Novel not found".to_string()))?;
    Ok(json!({
        "title": item["title"].as_str().unwrap_or_default(),
        "synopsis": item["story"].as_str().unwrap_or_default(),
        "page": item["page"].as_u64().unwrap_or(0),
    }))
}

pub async fn fetch_search(
    site: &SyosetuSite,
    client: &reqwest::Client,
    word: &str,
) -> Result<Value, AppError> {
    let data = site_api(
        site,
        client,
        &[
            ("of", OF_RANKING.to_string()),
            ("word", word.to_string()),
            ("lim", "20".to_string()),
            ("order", "hyoka".to_string()),
        ],
    )
    .await?;
    Ok(Value::Array(data))
}

pub async fn fetch_toc(
    site: &'static SyosetuSite,
    client: &reqwest::Client,
    ncode: &str,
) -> Result<Value, AppError> {
    let base_url = format!("{}/{}/", site.base_url, ncode);
    let res = with_headers(site, client.get(&base_url)).send().await?;
    if !res.status().is_success() {
        return Err(AppError::Upstream(format!(
            "{} toc error: {}",
            site.type_str,
            res.status()
        )));
    }
    let first = parse_toc(&res.text().await?);

    if first.last_page <= 1 {
        return Ok(toc_to_json(&first.title, &first.episodes));
    }

    let mut handles = Vec::new();
    for page in 2..=first.last_page {
        let client = client.clone();
        let url = format!("{}?p={}", base_url, page);
        handles.push(tokio::spawn(async move {
            let res = with_headers(site, client.get(&url)).send().await?;
            if !res.status().is_success() {
                return Err(AppError::Upstream(format!(
                    "{} toc page {} error: {}",
                    site.type_str,
                    page,
                    res.status()
                )));
            }
            let toc = parse_toc(&res.text().await?);
            Ok(toc.episodes)
        }));
    }

    let mut all_titles: Vec<String> = first.episodes.into_iter().map(|e| e.title).collect();
    for handle in handles {
        let episodes = handle
            .await
            .map_err(|e| AppError::Internal(e.to_string()))??;
        for ep in episodes {
            all_titles.push(ep.title);
        }
    }

    let episodes: Vec<Value> = all_titles
        .iter()
        .enumerate()
        .map(|(i, t)| json!({"num": i + 1, "title": t}))
        .collect();

    Ok(json!({
        "title": first.title,
        "episodes": episodes,
    }))
}

pub async fn fetch_page(
    site: &SyosetuSite,
    client: &reqwest::Client,
    ncode: &str,
    page: &str,
) -> Result<Option<String>, AppError> {
    let url = format!("{}/{}/{}/", site.base_url, ncode, page);
    let res = with_headers(site, client.get(&url)).send().await?;

    let res = if res.status().as_u16() == 404 {
        let fallback_url = format!("{}/{}/", site.base_url, ncode);
        with_headers(site, client.get(&fallback_url)).send().await?
    } else {
        res
    };

    if !res.status().is_success() {
        return Err(AppError::Upstream(format!(
            "{} page error: {}",
            site.type_str,
            res.status()
        )));
    }
    Ok(parse_page(&res.text().await?, ".p-novel__text"))
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

    // ── OF constants must use hyphens, not commas (regression for syosetu API bug) ──
    // The syosetu API `of` parameter requires hyphen-separated field codes.
    // Commas get URL-encoded to %2C by reqwest, causing the API to return null entries.

    #[test]
    fn of_ranking_uses_hyphens() {
        assert!(
            !OF_RANKING.contains(','),
            "OF_RANKING must not contain commas: {OF_RANKING}"
        );
        assert!(
            OF_RANKING.contains('-'),
            "OF_RANKING must use hyphen separators: {OF_RANKING}"
        );
    }

    #[test]
    fn of_datum_uses_hyphens() {
        assert!(
            !OF_DATUM.contains(','),
            "OF_DATUM must not contain commas: {OF_DATUM}"
        );
        assert!(
            OF_DATUM.contains('-'),
            "OF_DATUM must use hyphen separators: {OF_DATUM}"
        );
    }

    #[test]
    fn of_detail_uses_hyphens() {
        assert!(
            !OF_DETAIL.contains(','),
            "OF_DETAIL must not contain commas: {OF_DETAIL}"
        );
        assert!(
            OF_DETAIL.contains('-'),
            "OF_DETAIL must use hyphen separators: {OF_DETAIL}"
        );
    }
}
