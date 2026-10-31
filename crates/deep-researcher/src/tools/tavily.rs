use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Serialize)]
struct TavilyRequest<'a> {
    api_key: &'a str,
    query: &'a str,
    max_results: u32,
    search_depth: &'a str,
    include_answer: bool,
}

#[derive(Debug, Deserialize)]
pub struct TavilySearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    #[allow(dead_code)]
    pub score: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    results: Vec<TavilySearchResult>,
    #[allow(dead_code)]
    answer: Option<String>,
}

pub async fn search_tavily(
    query: &str,
    api_key: &str,
    max_results: u32,
) -> Result<Vec<TavilySearchResult>, String> {
    if api_key.is_empty() {
        return Err("Tavily API key is empty".to_string());
    }

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let payload = TavilyRequest {
        api_key,
        query,
        max_results,
        search_depth: "advanced",
        include_answer: true,
    };

    debug!("Sending Tavily search query: {}", query);

    let resp = client
        .post("https://api.tavily.com/search")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Tavily request error: {}", e))?;

    if !resp.status().is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        error!("Tavily error: {}", err_text);
        return Err(format!("Tavily returned error status: {}", err_text));
    }

    let parsed: TavilyResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Tavily response: {}", e))?;

    Ok(parsed.results)
}
