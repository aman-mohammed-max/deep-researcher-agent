use reqwest::header::USER_AGENT;
use reqwest::Client;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Clone, serde::Serialize)]
pub struct RedditPost {
    pub title: String,
    pub subreddit: String,
    pub url: String,
    pub permalink: String,
    pub selftext: String,
    pub score: i64,
    pub num_comments: i64,
}

pub async fn search_reddit(query: &str, limit: u32) -> Result<Vec<RedditPost>, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!(
        "https://www.reddit.com/search.json?q={}&sort=relevance&limit={}",
        urlencoding::encode(query),
        limit
    );

    debug!("Fetching Reddit public search: {}", url);

    let resp = client
        .get(&url)
        .header(USER_AGENT, "DeepResearcherBot/1.0 (Public Research)")
        .send()
        .await
        .map_err(|e| format!("Reddit connection error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Reddit returned HTTP status {}", resp.status()));
    }

    let val: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Reddit JSON: {}", e))?;

    let mut posts = Vec::new();
    if let Some(children) = val["data"]["children"].as_array() {
        for child in children {
            let data = &child["data"];
            let title = data["title"].as_str().unwrap_or("").to_string();
            let subreddit = data["subreddit"].as_str().unwrap_or("").to_string();
            let permalink = format!("https://reddit.com{}", data["permalink"].as_str().unwrap_or(""));
            let url = data["url"].as_str().unwrap_or(&permalink).to_string();
            let selftext = data["selftext"].as_str().unwrap_or("").to_string();
            let score = data["score"].as_i64().unwrap_or(0);
            let num_comments = data["num_comments"].as_i64().unwrap_or(0);

            if !title.is_empty() {
                posts.push(RedditPost {
                    title,
                    subreddit,
                    url,
                    permalink,
                    selftext: selftext.chars().take(800).collect(),
                    score,
                    num_comments,
                });
            }
        }
    }

    Ok(posts)
}
