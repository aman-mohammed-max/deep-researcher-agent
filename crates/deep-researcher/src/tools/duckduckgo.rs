use reqwest::header::USER_AGENT;
use reqwest::Client;
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DdgSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

pub async fn search_duckduckgo(query: &str, limit: u32) -> Result<Vec<DdgSearchResult>, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    // DuckDuckGo Instant Answer API
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding::encode(query)
    );

    let resp = client
        .get(&url)
        .header(USER_AGENT, "DeepResearcherBot/1.0")
        .send()
        .await
        .map_err(|e| format!("DuckDuckGo request error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("DuckDuckGo returned HTTP {}", resp.status()));
    }

    let val: Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    // Abstract text
    if let Some(abstract_text) = val["AbstractText"].as_str() {
        if !abstract_text.is_empty() {
            results.push(DdgSearchResult {
                title: val["Heading"].as_str().unwrap_or(query).to_string(),
                url: val["AbstractURL"].as_str().unwrap_or("https://duckduckgo.com").to_string(),
                snippet: abstract_text.to_string(),
            });
        }
    }

    // Related Topics
    if let Some(topics) = val["RelatedTopics"].as_array() {
        for t in topics.iter().take(limit as usize) {
            if let (Some(text), Some(u)) = (t["Text"].as_str(), t["FirstURL"].as_str()) {
                results.push(DdgSearchResult {
                    title: text.chars().take(80).collect(),
                    url: u.to_string(),
                    snippet: text.to_string(),
                });
            }
        }
    }

    Ok(results)
}
