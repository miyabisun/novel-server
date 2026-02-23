use crate::error::AppError;
use chrono::{DateTime, Utc};
use scraper::{Html, Selector};
use serde_json::{json, Map, Value};

const TYPE: &str = "kakuyomu";

fn parse_apollo_state(html: &str) -> Result<Value, AppError> {
    let doc = Html::parse_document(html);
    let sel =
        Selector::parse("#__NEXT_DATA__").map_err(|_| AppError::Internal("Bad selector".into()))?;
    let el = doc
        .select(&sel)
        .next()
        .ok_or_else(|| AppError::Upstream("Failed to parse kakuyomu work page".into()))?;
    let raw: String = el.text().collect();
    let json: Value = serde_json::from_str(&raw)?;
    json.get("props")
        .and_then(|p| p.get("pageProps"))
        .and_then(|pp| pp.get("__APOLLO_STATE__"))
        .cloned()
        .ok_or_else(|| AppError::Upstream("Apollo state not found".into()))
}

fn extract_work(apollo: &Value, id: &str) -> Result<WorkInfo, AppError> {
    let key = format!("Work:{}", id);
    let work = apollo
        .get(&key)
        .ok_or_else(|| AppError::Upstream("Work not found in Apollo state".into()))?;

    let title = work["title"].as_str().unwrap_or_default().to_string();
    let story = work["introduction"].as_str().unwrap_or_default().to_string();
    let novelupdated_at = work["lastEpisodePublishedAt"]
        .as_str()
        .and_then(|s| {
            s.parse::<DateTime<Utc>>()
                .ok()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        });

    Ok(WorkInfo {
        title,
        story,
        novelupdated_at,
    })
}

fn extract_episodes(apollo: &Value, _id: &str) -> Vec<EpisodeInfo> {
    let obj = match apollo.as_object() {
        Some(o) => o,
        None => return Vec::new(),
    };

    let mut pages = Vec::new();
    let mut num = 0u64;

    // Collect TOC chapter keys in order
    let mut toc_keys: Vec<&String> = obj
        .keys()
        .filter(|k| k.starts_with("TableOfContentsChapter"))
        .collect();
    toc_keys.sort();

    for toc_key in toc_keys {
        let chapter = &obj[toc_key];
        let episode_unions = match chapter.get("episodeUnions") {
            Some(Value::Array(arr)) => arr,
            _ => continue,
        };
        for ep_ref in episode_unions {
            let ref_str = match ep_ref.get("__ref").and_then(|r| r.as_str()) {
                Some(r) => r,
                None => continue,
            };
            let ep = match apollo.get(ref_str) {
                Some(e) => e,
                None => continue,
            };
            num += 1;
            pages.push(EpisodeInfo {
                num,
                id: ep["id"].as_str().unwrap_or_default().to_string(),
                title: ep["title"].as_str().unwrap_or_default().to_string(),
            });
        }
    }
    pages
}

pub async fn fetch_ranking(
    client: &reqwest::Client,
    genre: &str,
    rank_type: &str,
) -> Result<Vec<Value>, AppError> {
    let url = format!("https://kakuyomu.jp/rankings/{}/{}", genre, rank_type);
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Err(AppError::Upstream(format!(
            "kakuyomu ranking error: {}",
            res.status()
        )));
    }
    let doc = Html::parse_document(&res.text().await?);
    let work_sel = Selector::parse(".widget-work").unwrap();
    let title_sel = Selector::parse(".bookWalker-work-title").unwrap();
    let ep_count_sel = Selector::parse(".widget-workCard-episodeCount").unwrap();

    let mut result = Vec::new();
    for elem in doc.select(&work_sel) {
        let title_el = elem.select(&title_sel).next();
        let id = title_el
            .and_then(|el| el.value().attr("href"))
            .and_then(|href| href.rsplit('/').next())
            .unwrap_or_default();
        let title = title_el
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();
        let page_text = elem
            .select(&ep_count_sel)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();
        let page: u64 = page_text.replace("話", "").trim().parse().unwrap_or(0);

        result.push(json!({
            "id": id,
            "title": title,
            "page": page,
        }));
    }
    Ok(result)
}

pub async fn fetch_ranking_list(
    client: &reqwest::Client,
    period: &str,
) -> Result<Value, AppError> {
    if period == "quarter" {
        return Err(AppError::BadRequest(
            "kakuyomu does not support quarter ranking".to_string(),
        ));
    }
    let data = fetch_ranking(client, "all", period).await?;
    let mut result = Map::new();
    result.insert("総合".to_string(), Value::Array(data));
    Ok(Value::Object(result))
}

pub async fn fetch_search(client: &reqwest::Client, word: &str) -> Result<Value, AppError> {
    let url = format!(
        "https://kakuyomu.jp/search?q={}",
        urlencoding::encode(word)
    );
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Err(AppError::Upstream(format!(
            "kakuyomu search error: {}",
            res.status()
        )));
    }
    let apollo = parse_apollo_state(&res.text().await?)?;
    let obj = apollo.as_object().ok_or_else(|| {
        AppError::Upstream("Invalid Apollo state".into())
    })?;

    let mut results = Vec::new();
    for (key, val) in obj {
        if !key.starts_with("Work:") {
            continue;
        }
        let id = key.strip_prefix("Work:").unwrap_or_default();
        results.push(json!({
            "id": id,
            "title": val["title"].as_str().unwrap_or_default(),
            "page": val["publicEpisodeCount"].as_u64().unwrap_or(0),
        }));
    }
    Ok(Value::Array(results))
}

async fn fetch_work(client: &reqwest::Client, id: &str) -> Result<Value, AppError> {
    let url = format!("https://kakuyomu.jp/works/{}", id);
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Err(AppError::Upstream(format!(
            "kakuyomu work error: {}",
            res.status()
        )));
    }
    parse_apollo_state(&res.text().await?)
}

pub async fn fetch_toc(client: &reqwest::Client, id: &str) -> Result<Value, AppError> {
    let apollo = fetch_work(client, id).await?;
    let work = extract_work(&apollo, id)?;
    let episodes = extract_episodes(&apollo, id);
    let eps: Vec<Value> = episodes
        .iter()
        .map(|e| json!({"num": e.num, "title": e.title}))
        .collect();
    Ok(json!({
        "title": work.title,
        "episodes": eps,
    }))
}

pub async fn fetch_detail(client: &reqwest::Client, id: &str) -> Result<Value, AppError> {
    let apollo = fetch_work(client, id).await?;
    let work = extract_work(&apollo, id)?;
    let episodes = extract_episodes(&apollo, id);
    Ok(json!({
        "title": work.title,
        "synopsis": work.story,
        "page": episodes.len(),
    }))
}

pub async fn fetch_datum(client: &reqwest::Client, id: &str) -> Result<Value, AppError> {
    let apollo = fetch_work(client, id).await?;
    let work = extract_work(&apollo, id)?;
    let episodes = extract_episodes(&apollo, id);
    let pages: Vec<Value> = episodes
        .iter()
        .map(|e| {
            json!({
                "type": TYPE,
                "id": id,
                "num": e.num,
                "page_id": e.id,
                "title": e.title,
            })
        })
        .collect();
    let mut result = Map::new();
    result.insert("type".to_string(), json!(TYPE));
    result.insert("id".to_string(), json!(id));
    result.insert("title".to_string(), json!(work.title));
    result.insert("story".to_string(), json!(work.story));
    if let Some(ref dt) = work.novelupdated_at {
        result.insert("novelupdated_at".to_string(), json!(dt));
    }
    result.insert("pages".to_string(), Value::Array(pages));
    Ok(Value::Object(result))
}

pub async fn fetch_data(
    client: &reqwest::Client,
    ids: &[String],
) -> Result<Vec<Value>, AppError> {
    let mut results = Vec::new();
    for id in ids {
        results.push(fetch_datum(client, id).await?);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Ok(results)
}

pub async fn fetch_page(
    client: &reqwest::Client,
    id: &str,
    page_id: &str,
) -> Result<Option<String>, AppError> {
    let mut episode_id = page_id.to_string();

    // Small numbers are sequential page numbers that need resolution
    if let Ok(num) = page_id.parse::<u64>() {
        if num < 100_000 {
            let apollo = fetch_work(client, id).await?;
            let episodes = extract_episodes(&apollo, id);
            let ep = episodes.get((num as usize).wrapping_sub(1)).ok_or_else(|| {
                AppError::Upstream(format!("Episode {} not found", page_id))
            })?;
            episode_id = ep.id.clone();
        }
    }

    let url = format!(
        "https://kakuyomu.jp/works/{}/episodes/{}",
        id, episode_id
    );
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Err(AppError::Upstream(format!(
            "kakuyomu episode error: {}",
            res.status()
        )));
    }
    let doc = Html::parse_document(&res.text().await?);
    let sel = Selector::parse(".widget-episodeBody")
        .map_err(|_| AppError::Internal("Bad selector".into()))?;
    Ok(doc.select(&sel).next().map(|el| el.inner_html()))
}

struct WorkInfo {
    title: String,
    story: String,
    novelupdated_at: Option<String>,
}

struct EpisodeInfo {
    num: u64,
    id: String,
    title: String,
}
