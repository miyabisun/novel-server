use crate::error::AppError;
use scraper::{Html, Selector};
use serde_json::{json, Map, Value};

const TYPE: &str = "kakuyomu";

const RANKING_GENRES: &[(&str, &str)] = &[
    ("異世界ファンタジー", "fantasy"),
    ("現代ファンタジー", "action"),
    ("SF", "sf"),
    ("恋愛", "love_story"),
    ("ラブコメ", "romance"),
    ("現代ドラマ", "drama"),
    ("ホラー", "horror"),
];

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
    let story = work["introduction"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let novelupdated_at = work["lastEpisodePublishedAt"]
        .as_str()
        .filter(|value| crate::time::parse_upstream_timestamp(value).is_some())
        .map(str::to_string);

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

/// Extract the ranked work list from a ranking page's Apollo state.
/// Order comes from `ROOT_QUERY.rankedWorks(...).nodes`, which lists
/// `Work:` refs in rank order (kakuyomu Next entries use other typenames).
fn parse_ranking(apollo: &Value) -> Result<Vec<Value>, AppError> {
    let root = apollo
        .get("ROOT_QUERY")
        .and_then(|r| r.as_object())
        .ok_or_else(|| AppError::Upstream("ROOT_QUERY not found in kakuyomu ranking".into()))?;
    let nodes = root
        .iter()
        .find(|(key, _)| key.starts_with("rankedWorks("))
        .and_then(|(_, ranked)| ranked.get("nodes"))
        .and_then(|n| n.as_array())
        .ok_or_else(|| AppError::Upstream("rankedWorks not found in kakuyomu ranking".into()))?;

    let mut result = Vec::new();
    for node in nodes {
        let ref_str = match node.get("__ref").and_then(|r| r.as_str()) {
            Some(r) if r.starts_with("Work:") => r,
            _ => continue,
        };
        let work = match apollo.get(ref_str) {
            Some(w) => w,
            None => continue,
        };
        result.push(json!({
            "id": ref_str.strip_prefix("Work:").unwrap_or_default(),
            "title": work["title"].as_str().unwrap_or_default(),
            "page": work["publicEpisodeCount"].as_u64().unwrap_or(0),
        }));
    }
    Ok(result)
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
    let apollo = parse_apollo_state(&res.text().await?)?;
    parse_ranking(&apollo)
}

pub async fn fetch_ranking_list(client: &reqwest::Client, period: &str) -> Result<Value, AppError> {
    // Period support (kakuyomu lacks "quarter") is validated at the route layer
    // via ModuleType::supports_period before reaching here.
    let mut futures: Vec<_> = RANKING_GENRES
        .iter()
        .map(|(_, slug)| fetch_ranking(client, slug, period))
        .collect();
    // Fetch overall ranking (genre slug "all") as the last future
    futures.push(fetch_ranking(client, "all", period));
    let results = futures::future::join_all(futures).await;

    let mut map = Map::new();
    for (i, res) in results.into_iter().enumerate() {
        let data = res?;
        if i < RANKING_GENRES.len() {
            map.insert(RANKING_GENRES[i].0.to_string(), Value::Array(data));
        } else {
            map.insert("総合".to_string(), Value::Array(data));
        }
    }
    Ok(Value::Object(map))
}

pub async fn fetch_search(client: &reqwest::Client, word: &str) -> Result<Value, AppError> {
    let url = format!("https://kakuyomu.jp/search?q={}", urlencoding::encode(word));
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Err(AppError::Upstream(format!(
            "kakuyomu search error: {}",
            res.status()
        )));
    }
    let apollo = parse_apollo_state(&res.text().await?)?;
    let obj = apollo
        .as_object()
        .ok_or_else(|| AppError::Upstream("Invalid Apollo state".into()))?;

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

pub async fn fetch_data(client: &reqwest::Client, ids: &[String]) -> Result<Vec<Value>, AppError> {
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
            let ep = episodes
                .get((num as usize).wrapping_sub(1))
                .ok_or_else(|| AppError::Upstream(format!("Episode {} not found", page_id)))?;
            episode_id = ep.id.clone();
        }
    }

    let url = format!("https://kakuyomu.jp/works/{}/episodes/{}", id, episode_id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_next_data(apollo_state: Value) -> String {
        let next_data = json!({
            "props": {
                "pageProps": {
                    "__APOLLO_STATE__": apollo_state
                }
            }
        });
        format!(
            r#"<html><head><script id="__NEXT_DATA__" type="application/json">{}</script></head><body></body></html>"#,
            next_data
        )
    }

    #[test]
    fn parse_apollo_state_extracts_state() {
        let state = json!({"Work:123": {"title": "Test"}});
        let html = make_next_data(state.clone());
        let result = parse_apollo_state(&html).unwrap();
        assert_eq!(result, state);
    }

    #[test]
    fn parse_apollo_state_missing_next_data() {
        let html = "<html><body>no data</body></html>";
        assert!(parse_apollo_state(html).is_err());
    }

    #[test]
    fn parse_apollo_state_missing_apollo_key() {
        let html = r#"<html><head><script id="__NEXT_DATA__" type="application/json">{"props":{"pageProps":{}}}</script></head></html>"#;
        assert!(parse_apollo_state(html).is_err());
    }

    #[test]
    fn parse_ranking_maps_works_in_nodes_order() {
        // "Work:zzz" ranks first despite sorting after "Work:aaa" as a map key,
        // proving order comes from nodes, not Apollo object key order.
        let apollo = json!({
            "ROOT_QUERY": {
                "rankedWorks({\"first\":100,\"genre\":\"FANTASY\",\"period\":\"WEEKLY\"})": {
                    "nodes": [
                        {"__ref": "Work:zzz"},
                        {"__ref": "Work:aaa"}
                    ]
                }
            },
            "Work:zzz": {"title": "First", "publicEpisodeCount": 27},
            "Work:aaa": {"title": "Second", "publicEpisodeCount": 3}
        });
        let result = parse_ranking(&apollo).unwrap();
        assert_eq!(
            result,
            vec![
                json!({"id": "zzz", "title": "First", "page": 27}),
                json!({"id": "aaa", "title": "Second", "page": 3}),
            ]
        );
    }

    #[test]
    fn parse_ranking_skips_non_work_refs_and_missing_entries() {
        let apollo = json!({
            "ROOT_QUERY": {
                "rankedWorks({\"first\":100,\"genre\":null,\"period\":\"WEEKLY\"})": {
                    "nodes": [
                        {"__ref": "KakuyomuNextWork:next1"},
                        {"__ref": "Work:missing"},
                        {"__ref": "Work:ok"}
                    ]
                }
            },
            "KakuyomuNextWork:next1": {"title": "Next", "publicEpisodeCount": 1},
            "Work:ok": {"title": "Kept", "publicEpisodeCount": 5}
        });
        let result = parse_ranking(&apollo).unwrap();
        assert_eq!(
            result,
            vec![json!({"id": "ok", "title": "Kept", "page": 5})]
        );
    }

    #[test]
    fn parse_ranking_errors_when_ranked_works_absent() {
        // A schema change must surface as an error, not a silently empty list.
        let apollo = json!({"ROOT_QUERY": {"visitor": null}});
        assert!(parse_ranking(&apollo).is_err());
        let no_root = json!({"Work:abc": {"title": "T"}});
        assert!(parse_ranking(&no_root).is_err());
    }

    #[test]
    fn extract_work_basic() {
        let apollo = json!({
            "Work:abc": {
                "title": "My Novel",
                "introduction": "A great story",
                "lastEpisodePublishedAt": "2025-01-15T10:30:00Z"
            }
        });
        let work = extract_work(&apollo, "abc").unwrap();
        assert_eq!(work.title, "My Novel");
        assert_eq!(work.story, "A great story");
        assert_eq!(
            work.novelupdated_at,
            Some("2025-01-15T10:30:00Z".to_string())
        );
    }

    #[test]
    fn extract_work_missing_id() {
        let apollo = json!({"Work:other": {"title": "X"}});
        assert!(extract_work(&apollo, "abc").is_err());
    }

    #[test]
    fn extract_work_missing_optional_fields() {
        let apollo = json!({
            "Work:abc": {}
        });
        let work = extract_work(&apollo, "abc").unwrap();
        assert_eq!(work.title, "");
        assert_eq!(work.story, "");
        assert!(work.novelupdated_at.is_none());
    }

    #[test]
    fn extract_work_invalid_date() {
        let apollo = json!({
            "Work:abc": {
                "title": "T",
                "lastEpisodePublishedAt": "not-a-date"
            }
        });
        let work = extract_work(&apollo, "abc").unwrap();
        assert!(work.novelupdated_at.is_none());
    }

    #[test]
    fn extract_episodes_basic() {
        let apollo = json!({
            "TableOfContentsChapter:ch1": {
                "episodeUnions": [
                    {"__ref": "Episode:ep1"},
                    {"__ref": "Episode:ep2"}
                ]
            },
            "Episode:ep1": {"id": "ep1", "title": "Chapter 1"},
            "Episode:ep2": {"id": "ep2", "title": "Chapter 2"}
        });
        let episodes = extract_episodes(&apollo, "abc");
        assert_eq!(episodes.len(), 2);
        assert_eq!(episodes[0].num, 1);
        assert_eq!(episodes[0].id, "ep1");
        assert_eq!(episodes[0].title, "Chapter 1");
        assert_eq!(episodes[1].num, 2);
        assert_eq!(episodes[1].id, "ep2");
    }

    #[test]
    fn extract_episodes_sorted_by_chapter_key() {
        let apollo = json!({
            "TableOfContentsChapter:ch2": {
                "episodeUnions": [{"__ref": "Episode:ep3"}]
            },
            "TableOfContentsChapter:ch1": {
                "episodeUnions": [{"__ref": "Episode:ep1"}, {"__ref": "Episode:ep2"}]
            },
            "Episode:ep1": {"id": "ep1", "title": "Ep 1"},
            "Episode:ep2": {"id": "ep2", "title": "Ep 2"},
            "Episode:ep3": {"id": "ep3", "title": "Ep 3"}
        });
        let episodes = extract_episodes(&apollo, "abc");
        assert_eq!(episodes.len(), 3);
        // ch1 episodes come first since keys are sorted
        assert_eq!(episodes[0].title, "Ep 1");
        assert_eq!(episodes[1].title, "Ep 2");
        assert_eq!(episodes[2].title, "Ep 3");
    }

    #[test]
    fn extract_episodes_empty_when_no_chapters() {
        let apollo = json!({"Work:abc": {"title": "T"}});
        let episodes = extract_episodes(&apollo, "abc");
        assert!(episodes.is_empty());
    }

    #[test]
    fn extract_episodes_skips_missing_refs() {
        let apollo = json!({
            "TableOfContentsChapter:ch1": {
                "episodeUnions": [
                    {"__ref": "Episode:exists"},
                    {"__ref": "Episode:missing"}
                ]
            },
            "Episode:exists": {"id": "e1", "title": "Found"}
        });
        let episodes = extract_episodes(&apollo, "abc");
        assert_eq!(episodes.len(), 1);
        assert_eq!(episodes[0].id, "e1");
    }

    #[test]
    fn extract_episodes_non_object_apollo() {
        let apollo = json!("not an object");
        let episodes = extract_episodes(&apollo, "abc");
        assert!(episodes.is_empty());
    }
}
