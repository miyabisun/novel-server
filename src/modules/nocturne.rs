use crate::error::AppError;
use crate::modules::syosetu;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde_json::{json, Value};

const API_URL: &str = "https://api.syosetu.com/novel18api/api/";
const BASE_URL: &str = "https://novel18.syosetu.com";
const TYPE: &str = "nocturne";

/// Output fields for ranking/search (title, writer, ncode, general_all_no, noveltype)
pub const OF_RANKING: &str = "t-w-n-ga-nt";
/// Output fields for datum/data (ncode, title, general_all_no, story, novelupdated_at)
pub const OF_DATUM: &str = "n-t-ga-s-nu";
/// Output fields for detail (title, story, general_all_no)
pub const OF_DETAIL: &str = "t-s-ga";

fn over18_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_static("over18=yes"));
    headers
}

fn to_datum(datum: &Value) -> Value {
    let id = datum["id"].as_str().unwrap_or_default();
    let page_count = datum["page"].as_u64().unwrap_or(0);
    let pages = syosetu::build_pages(TYPE, id, page_count);
    let mut obj = datum.as_object().cloned().unwrap_or_default();
    obj.remove("page");
    obj.insert("type".to_string(), json!(TYPE));
    obj.insert("pages".to_string(), pages);
    Value::Object(obj)
}

async fn fetch_api(
    client: &reqwest::Client,
    params: &[(&str, String)],
) -> Result<Vec<Value>, AppError> {
    syosetu::fetch_api(client, API_URL, params, None).await
}

pub async fn fetch_ranking(
    client: &reqwest::Client,
    genre: u32,
    limit: usize,
    order: &str,
) -> Result<Vec<Value>, AppError> {
    fetch_api(
        client,
        &[
            ("of", OF_RANKING.to_string()),
            ("lim", limit.to_string()),
            ("order", order.to_string()),
            ("nocgenre", genre.to_string()),
        ],
    )
    .await
}

pub async fn fetch_ranking_list(
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
    let data = fetch_ranking(client, 1, limit, order).await?;
    let mut result = serde_json::Map::new();
    result.insert("ノクターン".to_string(), Value::Array(data));
    Ok(Value::Object(result))
}

pub async fn fetch_datum(client: &reqwest::Client, id: &str) -> Result<Value, AppError> {
    let data = fetch_api(
        client,
        &[
            ("of", OF_DATUM.to_string()),
            ("ncode", id.to_string()),
        ],
    )
    .await?;
    data.first()
        .map(to_datum)
        .ok_or_else(|| AppError::Upstream("Novel not found".to_string()))
}

pub async fn fetch_data(
    client: &reqwest::Client,
    ids: &[String],
) -> Result<Vec<Value>, AppError> {
    let ncode_str = ids.join("-");
    let data = fetch_api(
        client,
        &[
            ("of", OF_DATUM.to_string()),
            ("ncode", ncode_str),
        ],
    )
    .await?;
    Ok(data.iter().map(to_datum).collect())
}

pub async fn fetch_detail(client: &reqwest::Client, id: &str) -> Result<Value, AppError> {
    let data = fetch_api(
        client,
        &[
            ("of", OF_DETAIL.to_string()),
            ("ncode", id.to_string()),
        ],
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

pub async fn fetch_search(client: &reqwest::Client, word: &str) -> Result<Value, AppError> {
    let data = fetch_api(
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

pub async fn fetch_toc(client: &reqwest::Client, ncode: &str) -> Result<Value, AppError> {
    let base_url = format!("{}/{}/", BASE_URL, ncode);
    let headers = over18_headers();
    let res = client
        .get(&base_url)
        .headers(headers.clone())
        .send()
        .await?;
    if !res.status().is_success() {
        return Err(AppError::Upstream(format!(
            "nocturne toc error: {}",
            res.status()
        )));
    }
    let first = syosetu::parse_toc(&res.text().await?);

    if first.last_page <= 1 {
        return Ok(toc_to_json(&first.title, &first.episodes));
    }

    let mut handles = Vec::new();
    for page in 2..=first.last_page {
        let client = client.clone();
        let url = format!("{}?p={}", base_url, page);
        let hdrs = headers.clone();
        handles.push(tokio::spawn(async move {
            let res = client.get(&url).headers(hdrs).send().await?;
            if !res.status().is_success() {
                return Err(AppError::Upstream(format!(
                    "nocturne toc page {} error: {}",
                    page,
                    res.status()
                )));
            }
            let toc = syosetu::parse_toc(&res.text().await?);
            Ok(toc.episodes)
        }));
    }

    let mut all_titles: Vec<String> = first.episodes.into_iter().map(|e| e.title).collect();
    for handle in handles {
        let episodes = handle.await.map_err(|e| AppError::Internal(e.to_string()))??;
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
    client: &reqwest::Client,
    ncode: &str,
    page: &str,
) -> Result<Option<String>, AppError> {
    let headers = over18_headers();
    let url = format!("{}/{}/{}/", BASE_URL, ncode, page);
    let res = client.get(&url).headers(headers.clone()).send().await?;

    let res = if res.status().as_u16() == 404 {
        let fallback_url = format!("{}/{}/", BASE_URL, ncode);
        client
            .get(&fallback_url)
            .headers(headers)
            .send()
            .await?
    } else {
        res
    };

    if !res.status().is_success() {
        return Err(AppError::Upstream(format!(
            "nocturne page error: {}",
            res.status()
        )));
    }
    Ok(syosetu::parse_page(&res.text().await?, ".p-novel__text"))
}

fn toc_to_json(title: &str, episodes: &[syosetu::TocEpisode]) -> Value {
    let eps: Vec<Value> = episodes
        .iter()
        .map(|e| json!({"num": e.num, "title": e.title}))
        .collect();
    json!({
        "title": title,
        "episodes": eps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── OF constants must use hyphens, not commas (regression for syosetu API bug) ──
    // The syosetu API `of` parameter requires hyphen-separated field codes.
    // Commas get URL-encoded to %2C by reqwest, causing the API to return null entries.

    #[test]
    fn of_ranking_uses_hyphens() {
        assert!(!OF_RANKING.contains(','), "OF_RANKING must not contain commas: {OF_RANKING}");
        assert!(OF_RANKING.contains('-'), "OF_RANKING must use hyphen separators: {OF_RANKING}");
    }

    #[test]
    fn of_datum_uses_hyphens() {
        assert!(!OF_DATUM.contains(','), "OF_DATUM must not contain commas: {OF_DATUM}");
        assert!(OF_DATUM.contains('-'), "OF_DATUM must use hyphen separators: {OF_DATUM}");
    }

    #[test]
    fn of_detail_uses_hyphens() {
        assert!(!OF_DETAIL.contains(','), "OF_DETAIL must not contain commas: {OF_DETAIL}");
        assert!(OF_DETAIL.contains('-'), "OF_DETAIL must use hyphen separators: {OF_DETAIL}");
    }
}
