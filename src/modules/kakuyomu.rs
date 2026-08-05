use crate::error::AppError;
use scraper::{Html, Selector};
use serde_json::{json, Map, Value};

const TYPE: &str = "kakuyomu";

// kakuyomu serves its site data from an unauthenticated GraphQL endpoint
// (introspection is disabled, so field names here come from observed traffic).
// Episode bodies are NOT exposed over GraphQL — fetch_page still scrapes HTML.
const GRAPHQL_URL: &str = "https://kakuyomu.jp/graphql";

const RANKING_GENRES: &[(&str, &str)] = &[
    ("異世界ファンタジー", "fantasy"),
    ("現代ファンタジー", "action"),
    ("SF", "sf"),
    ("恋愛", "love_story"),
    ("ラブコメ", "romance"),
    ("現代ドラマ", "drama"),
    ("ホラー", "horror"),
];

const WORK_QUERY: &str = "query($id: ID!) { work(id: $id) { \
    title introduction lastEpisodePublishedAt \
    tableOfContents { episodeUnions { __typename ... on Episode { id title } } } } }";

const SEARCH_QUERY: &str = "query($q: String!) { \
    searchWorks(first: 20, query: $q) { nodes { id title publicEpisodeCount } } }";

/// Genre slugs and periods come from fixed whitelists (RANKING_GENRES and the
/// route layer's VALID_PERIODS), and map to GraphQL enums by uppercasing.
/// "all" means the overall ranking, which omits the genre argument.
fn ranking_query(genre: &str, period: &str) -> String {
    let genre_arg = if genre == "all" {
        String::new()
    } else {
        format!(", genre: {}", genre.to_ascii_uppercase())
    };
    format!(
        "query {{ rankedWorks(first: 100, period: {}{}) {{ nodes {{ id title publicEpisodeCount }} }} }}",
        period.to_ascii_uppercase(),
        genre_arg
    )
}

async fn graphql(
    client: &reqwest::Client,
    query: &str,
    variables: Value,
) -> Result<Value, AppError> {
    let res = client
        .post(GRAPHQL_URL)
        .json(&json!({"query": query, "variables": variables}))
        .send()
        .await?;
    if !res.status().is_success() {
        return Err(AppError::Upstream(format!(
            "kakuyomu graphql error: {}",
            res.status()
        )));
    }
    extract_data(res.json().await?)
}

/// A 200 response can still carry failures; never treat those as success.
fn extract_data(body: Value) -> Result<Value, AppError> {
    if let Some(first) = body
        .get("errors")
        .and_then(Value::as_array)
        .and_then(|e| e.first())
    {
        return Err(AppError::Upstream(format!(
            "kakuyomu graphql error: {}",
            first["message"].as_str().unwrap_or("unknown")
        )));
    }
    match body.get("data") {
        Some(data) if !data.is_null() => Ok(data.clone()),
        _ => Err(AppError::Upstream(
            "kakuyomu graphql response has no data".into(),
        )),
    }
}

/// Required-field readers: a missing or mistyped field means the schema
/// drifted, and must fail loudly. Defaulting instead would let the background
/// sync overwrite good favorite metadata with blanks and zeros.
fn require_str<'a>(node: &'a Value, field: &str) -> Result<&'a str, AppError> {
    node.get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Upstream(format!("kakuyomu response missing {}", field)))
}

fn require_u64(node: &Value, field: &str) -> Result<u64, AppError> {
    node.get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| AppError::Upstream(format!("kakuyomu response missing {}", field)))
}

fn work_summary(node: &Value) -> Result<Value, AppError> {
    Ok(json!({
        "id": require_str(node, "id")?,
        "title": require_str(node, "title")?,
        "page": require_u64(node, "publicEpisodeCount")?,
    }))
}

fn parse_ranking(data: &Value) -> Result<Vec<Value>, AppError> {
    let nodes = data
        .get("rankedWorks")
        .and_then(|r| r.get("nodes"))
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Upstream("rankedWorks not found in kakuyomu response".into()))?;
    // Every genre normally ranks 100 works; an empty list means upstream broke
    if nodes.is_empty() {
        return Err(AppError::Upstream(
            "kakuyomu ranking returned no works".into(),
        ));
    }
    nodes.iter().map(work_summary).collect()
}

fn parse_search(data: &Value) -> Result<Vec<Value>, AppError> {
    let nodes = data
        .get("searchWorks")
        .and_then(|r| r.get("nodes"))
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Upstream("searchWorks not found in kakuyomu response".into()))?;
    // Zero hits is a legitimate search result, so an empty list stays Ok here
    nodes.iter().map(work_summary).collect()
}

/// Episodes come flattened from `tableOfContents` chapters in reading order.
/// Union members other than Episode (if any appear) are skipped.
fn parse_work(data: &Value) -> Result<(WorkInfo, Vec<EpisodeInfo>), AppError> {
    let work = match data.get("work") {
        Some(w) if !w.is_null() => w,
        _ => return Err(AppError::Upstream("Work not found".into())),
    };

    // introduction and lastEpisodePublishedAt are genuinely optional upstream;
    // everything else read here is required
    let info = WorkInfo {
        title: require_str(work, "title")?.to_string(),
        story: work["introduction"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        novelupdated_at: work["lastEpisodePublishedAt"]
            .as_str()
            .filter(|value| crate::time::parse_upstream_timestamp(value).is_some())
            .map(str::to_string),
    };

    let chapters = work
        .get("tableOfContents")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Upstream("kakuyomu response missing tableOfContents".into()))?;

    let mut episodes = Vec::new();
    let mut num = 0u64;
    for chapter in chapters {
        let unions = chapter
            .get("episodeUnions")
            .and_then(Value::as_array)
            .ok_or_else(|| AppError::Upstream("kakuyomu response missing episodeUnions".into()))?;
        for ep in unions {
            if ep["__typename"].as_str() != Some("Episode") {
                continue;
            }
            num += 1;
            episodes.push(EpisodeInfo {
                num,
                id: require_str(ep, "id")?.to_string(),
                title: require_str(ep, "title")?.to_string(),
            });
        }
    }
    Ok((info, episodes))
}

pub async fn fetch_ranking(
    client: &reqwest::Client,
    genre: &str,
    rank_type: &str,
) -> Result<Vec<Value>, AppError> {
    let data = graphql(client, &ranking_query(genre, rank_type), json!({})).await?;
    parse_ranking(&data)
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
    let data = graphql(client, SEARCH_QUERY, json!({"q": word})).await?;
    Ok(Value::Array(parse_search(&data)?))
}

async fn fetch_work_data(
    client: &reqwest::Client,
    id: &str,
) -> Result<(WorkInfo, Vec<EpisodeInfo>), AppError> {
    let data = graphql(client, WORK_QUERY, json!({"id": id})).await?;
    parse_work(&data)
}

pub async fn fetch_toc(client: &reqwest::Client, id: &str) -> Result<Value, AppError> {
    let (work, episodes) = fetch_work_data(client, id).await?;
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
    let (work, episodes) = fetch_work_data(client, id).await?;
    Ok(json!({
        "title": work.title,
        "synopsis": work.story,
        "page": episodes.len(),
    }))
}

pub async fn fetch_datum(client: &reqwest::Client, id: &str) -> Result<Value, AppError> {
    let (work, episodes) = fetch_work_data(client, id).await?;
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
            let (_, episodes) = fetch_work_data(client, id).await?;
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
    // Episode bodies are only published as HTML; this selector is the one
    // remaining scraping dependency on kakuyomu's page markup.
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

    #[test]
    fn extract_data_returns_data() {
        let body = json!({"data": {"work": {"title": "T"}}});
        assert_eq!(extract_data(body).unwrap(), json!({"work": {"title": "T"}}));
    }

    #[test]
    fn extract_data_errors_take_precedence_over_data() {
        // kakuyomu returns errors alongside "data": {"work": null} for a
        // missing work; the error must win over the partial data.
        let body = json!({
            "errors": [{"message": "Not found: xyz"}],
            "data": {"work": null}
        });
        let err = extract_data(body).unwrap_err();
        assert!(format!("{:?}", err).contains("Not found: xyz"));
    }

    #[test]
    fn extract_data_missing_or_null_data_is_error() {
        assert!(extract_data(json!({})).is_err());
        assert!(extract_data(json!({"data": null})).is_err());
    }

    #[test]
    fn ranking_query_uppercases_enums_and_omits_genre_for_all() {
        let q = ranking_query("love_story", "weekly");
        assert!(q.contains("period: WEEKLY"));
        assert!(q.contains("genre: LOVE_STORY"));
        let overall = ranking_query("all", "daily");
        assert!(overall.contains("period: DAILY"));
        assert!(!overall.contains("genre:"));
    }

    #[test]
    fn parse_ranking_maps_nodes_in_order() {
        let data = json!({
            "rankedWorks": {
                "nodes": [
                    {"id": "zzz", "title": "First", "publicEpisodeCount": 27},
                    {"id": "aaa", "title": "Second", "publicEpisodeCount": 3}
                ]
            }
        });
        let result = parse_ranking(&data).unwrap();
        assert_eq!(
            result,
            vec![
                json!({"id": "zzz", "title": "First", "page": 27}),
                json!({"id": "aaa", "title": "Second", "page": 3}),
            ]
        );
    }

    #[test]
    fn parse_ranking_errors_when_ranked_works_absent_or_empty() {
        // A schema change must surface as an error, not a silently empty list.
        assert!(parse_ranking(&json!({})).is_err());
        assert!(parse_ranking(&json!({"rankedWorks": {"nodes": null}})).is_err());
        // Genres always rank works, so an empty list is a failure too
        assert!(parse_ranking(&json!({"rankedWorks": {"nodes": []}})).is_err());
    }

    #[test]
    fn parse_ranking_errors_on_malformed_node() {
        // A node missing a required field must not degrade to ""/0 — the
        // background sync would propagate those blanks into stored favorites.
        let missing_title = json!({
            "rankedWorks": {"nodes": [{"id": "a", "publicEpisodeCount": 5}]}
        });
        assert!(parse_ranking(&missing_title).is_err());
        let mistyped_count = json!({
            "rankedWorks": {"nodes": [{"id": "a", "title": "T", "publicEpisodeCount": "5"}]}
        });
        assert!(parse_ranking(&mistyped_count).is_err());
    }

    #[test]
    fn parse_search_preserves_order_and_allows_empty() {
        let data = json!({
            "searchWorks": {
                "nodes": [
                    {"id": "b", "title": "Best match", "publicEpisodeCount": 5},
                    {"id": "a", "title": "Second match", "publicEpisodeCount": 178}
                ]
            }
        });
        let result = parse_search(&data).unwrap();
        assert_eq!(result[0]["id"], "b");
        assert_eq!(result[1]["page"], 178);

        // Zero hits is a legitimate result, not an upstream failure
        let empty = parse_search(&json!({"searchWorks": {"nodes": []}})).unwrap();
        assert!(empty.is_empty());

        assert!(parse_search(&json!({})).is_err());
    }

    #[test]
    fn parse_work_extracts_metadata_and_flattens_chapters_in_order() {
        let data = json!({
            "work": {
                "title": "My Novel",
                "introduction": "A great story",
                "lastEpisodePublishedAt": "2025-01-15T10:30:00Z",
                "tableOfContents": [
                    {"episodeUnions": [
                        {"__typename": "Episode", "id": "ep1", "title": "Chapter 1"},
                        {"__typename": "Episode", "id": "ep2", "title": "Chapter 2"}
                    ]},
                    {"episodeUnions": [
                        {"__typename": "Episode", "id": "ep3", "title": "Chapter 3"}
                    ]}
                ]
            }
        });
        let (work, episodes) = parse_work(&data).unwrap();
        assert_eq!(work.title, "My Novel");
        assert_eq!(work.story, "A great story");
        assert_eq!(
            work.novelupdated_at,
            Some("2025-01-15T10:30:00Z".to_string())
        );
        assert_eq!(episodes.len(), 3);
        assert_eq!(episodes[0].num, 1);
        assert_eq!(episodes[0].id, "ep1");
        assert_eq!(episodes[2].num, 3);
        assert_eq!(episodes[2].title, "Chapter 3");
    }

    #[test]
    fn parse_work_skips_non_episode_unions_without_numbering_them() {
        let data = json!({
            "work": {
                "title": "T",
                "tableOfContents": [
                    {"episodeUnions": [
                        {"__typename": "SomethingElse"},
                        {"__typename": "Episode", "id": "ep1", "title": "Only"}
                    ]}
                ]
            }
        });
        let (_, episodes) = parse_work(&data).unwrap();
        assert_eq!(episodes.len(), 1);
        assert_eq!(episodes[0].num, 1);
        assert_eq!(episodes[0].id, "ep1");
    }

    #[test]
    fn parse_work_missing_work_is_error() {
        assert!(parse_work(&json!({})).is_err());
        assert!(parse_work(&json!({"work": null})).is_err());
    }

    #[test]
    fn parse_work_tolerates_missing_optional_fields() {
        // Only introduction and lastEpisodePublishedAt are optional
        let data = json!({"work": {"title": "T", "tableOfContents": []}});
        let (work, episodes) = parse_work(&data).unwrap();
        assert_eq!(work.title, "T");
        assert_eq!(work.story, "");
        assert!(work.novelupdated_at.is_none());
        assert!(episodes.is_empty());
    }

    #[test]
    fn parse_work_errors_on_missing_required_fields() {
        // No title
        assert!(parse_work(&json!({"work": {"tableOfContents": []}})).is_err());
        // No tableOfContents
        assert!(parse_work(&json!({"work": {"title": "T"}})).is_err());
        // Chapter without episodeUnions
        assert!(parse_work(&json!({
            "work": {"title": "T", "tableOfContents": [{}]}
        }))
        .is_err());
        // Episode without id
        assert!(parse_work(&json!({
            "work": {"title": "T", "tableOfContents": [
                {"episodeUnions": [{"__typename": "Episode", "title": "no id"}]}
            ]}
        }))
        .is_err());
    }

    #[test]
    fn parse_work_drops_invalid_date() {
        let data = json!({
            "work": {
                "title": "T",
                "tableOfContents": [],
                "lastEpisodePublishedAt": "not-a-date"
            }
        });
        let (work, _) = parse_work(&data).unwrap();
        assert!(work.novelupdated_at.is_none());
    }
}
