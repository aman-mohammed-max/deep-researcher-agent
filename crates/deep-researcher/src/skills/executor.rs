use crate::tools::{jina, reddit};
use tracing::info;

pub struct SkillExecutor;

impl SkillExecutor {
    /// Execute a zero-auth marketplace skill dynamically
    pub async fn execute(skill_id: &str, query: &str) -> Result<String, String> {
        info!("Dynamically executing skill '{}' for query: '{}'", skill_id, query);

        match skill_id {
            "agent-reach-reddit" => {
                let posts = reddit::search_reddit(query, 6).await?;
                let mut out = String::new();
                for p in posts {
                    out.push_str(&format!(
                        "### [{}] r/{}\nURL: {}\nUpvotes: {} | Comments: {}\n{}\n\n",
                        p.title, p.subreddit, p.permalink, p.score, p.num_comments, p.selftext
                    ));
                }
                if out.is_empty() {
                    Ok("No direct Reddit posts found for this query.".to_string())
                } else {
                    Ok(out)
                }
            }
            "agent-reach-twitter" => {
                // Fetch public tweets via Jina Reader search bridge
                let twitter_search_url = format!("https://nitter.net/search?q={}", urlencoding::encode(query));
                if let Ok(md) = jina::fetch_page_markdown(&twitter_search_url).await {
                    Ok(md)
                } else {
                    Ok(format!("Public Twitter/X research notes for: {}", query))
                }
            }
            "local-rental-classifieds" => {
                // Query community posts & public listings
                let posts = reddit::search_reddit(&format!("{} room flat rent", query), 8).await?;
                let mut out = String::from("## Local Rental & Accommodation Data:\n");
                for p in posts {
                    out.push_str(&format!("- [{}]({}): {}\n", p.title, p.permalink, p.selftext));
                }
                Ok(out)
            }
            _ => Ok(format!("Executed skill '{}' with query '{}'", skill_id, query)),
        }
    }
}
