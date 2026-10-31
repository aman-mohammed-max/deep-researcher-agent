pub mod duckduckgo;
pub mod jina;
pub mod reddit;
pub mod tavily;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source_type: String, // "web", "reddit", "news", "direct_page"
}

#[derive(Clone)]
pub struct SearchDispatcher {
    pub tavily_key: Option<String>,
}

impl SearchDispatcher {
    pub fn new(tavily_key: Option<String>) -> Self {
        Self { tavily_key }
    }

    /// Primary search dispatch with intelligent multi-source fallback
    pub async fn search(&self, query: &str, target_angle: &str, limit: u32) -> Vec<UnifiedSearchResult> {
        let mut results = Vec::new();

        // 1. If community sentiment / local angle, query Reddit first
        if target_angle == "community_sentiment" || target_angle == "pricing_local" || query.contains("reddit") {
            if let Ok(posts) = reddit::search_reddit(query, limit).await {
                for p in posts {
                    results.push(UnifiedSearchResult {
                        title: format!("[Reddit r/{}] {}", p.subreddit, p.title),
                        url: p.permalink,
                        snippet: p.selftext,
                        source_type: "reddit".to_string(),
                    });
                }
            }
        }

        // 2. Query Tavily if API key is provided
        if results.is_empty() {
            if let Some(ref key) = self.tavily_key {
                if let Ok(tavily_res) = tavily::search_tavily(query, key, limit).await {
                    for t in tavily_res {
                        results.push(UnifiedSearchResult {
                            title: t.title,
                            url: t.url,
                            snippet: t.content,
                            source_type: "web".to_string(),
                        });
                    }
                }
            }
        }

        // 3. Fallback to DuckDuckGo if still empty
        if results.is_empty() {
            if let Ok(ddg_res) = duckduckgo::search_duckduckgo(query, limit).await {
                for d in ddg_res {
                    results.push(UnifiedSearchResult {
                        title: d.title,
                        url: d.url,
                        snippet: d.snippet,
                        source_type: "web".to_string(),
                    });
                }
            }
        }

        // 4. Try Reddit fallback if web returned nothing
        if results.is_empty() {
            if let Ok(posts) = reddit::search_reddit(query, limit).await {
                for p in posts {
                    results.push(UnifiedSearchResult {
                        title: format!("[Reddit r/{}] {}", p.subreddit, p.title),
                        url: p.permalink,
                        snippet: p.selftext,
                        source_type: "reddit".to_string(),
                    });
                }
            }
        }

        results
    }
}
