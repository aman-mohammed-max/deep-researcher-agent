use reqwest::Client;
use tracing::debug;

pub async fn fetch_page_markdown(url: &str) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let jina_url = format!("https://r.jina.ai/{}", url);
    debug!("Extracting markdown via Jina Reader: {}", jina_url);

    let resp = client
        .get(&jina_url)
        .header("X-Return-Format", "markdown")
        .header("User-Agent", "Mozilla/5.0 DeepResearcher/1.0")
        .send()
        .await
        .map_err(|e| format!("Jina reader request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Jina Reader returned HTTP {}", resp.status()));
    }

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response text: {}", e))?;

    // Trim to 8000 characters to prevent context explosion
    let trimmed: String = text.chars().take(8000).collect();
    Ok(trimmed)
}
