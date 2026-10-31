use crate::agents::state::{ResearchNote, SourceEntry, SubQuery};
use crate::error::AppError;
use crate::llm::{ChatMessage, LlmClient, TokenUsage};
use crate::prompts::PromptEngine;
use crate::skills::{MarketplaceRegistry, SkillExecutor};
use crate::tools::SearchDispatcher;
use tracing::info;

pub struct ResearchWorker<'a> {
    llm: &'a LlmClient,
    prompts: &'a PromptEngine,
    searcher: &'a SearchDispatcher,
    marketplace: &'a MarketplaceRegistry,
}

impl<'a> ResearchWorker<'a> {
    pub fn new(
        llm: &'a LlmClient,
        prompts: &'a PromptEngine,
        searcher: &'a SearchDispatcher,
        marketplace: &'a MarketplaceRegistry,
    ) -> Self {
        Self {
            llm,
            prompts,
            searcher,
            marketplace,
        }
    }

    /// Execute a sub-query with adaptive tool execution & reactive skill marketplace escalation
    pub async fn execute(&self, sub_query: &SubQuery) -> Result<(ResearchNote, TokenUsage), AppError> {
        info!("Executing research worker for query: '{}' (angle: {})", sub_query.query, sub_query.target_angle);

        // 1. Primary standard search dispatch
        let search_results = self.searcher.search(&sub_query.query, &sub_query.target_angle, 6).await;
        let mut executed_skills = Vec::new();

        // 2. ADAPTIVE ESCALATION: If information is limited (fewer than 2 results) OR specialized angle
        let needs_specialized_skill = search_results.len() < 2
            || sub_query.target_angle == "community_sentiment"
            || sub_query.target_angle == "pricing_local"
            || sub_query.query.to_lowercase().contains("room")
            || sub_query.query.to_lowercase().contains("flat")
            || sub_query.query.to_lowercase().contains("stock");

        if needs_specialized_skill {
            info!("Information deficit or specialized domain detected. Checking Skills Marketplace...");
            let candidate_skills = self.marketplace.search_skills(&sub_query.query).await;

            for skill in candidate_skills.iter().take(2) {
                // Verified zero-auth & quality check (already enforced by registry)
                info!("Selected top-rated marketplace skill: '{}' (⭐ {} stars)", skill.name, skill.stars);

                if let Ok(skill_data) = SkillExecutor::execute(&skill.id, &sub_query.query).await {
                    executed_skills.push(format!("### [Marketplace Skill: {} (⭐ {})]\n{}", skill.name, skill.stars, skill_data));
                }
            }
        }

        // 3. Aggregate all findings
        let mut combined_content = String::new();
        let mut sources = Vec::new();

        for res in &search_results {
            combined_content.push_str(&format!(
                "Title: {}\nURL: {}\nSnippet: {}\n\n",
                res.title, res.url, res.snippet
            ));
            sources.push(SourceEntry {
                title: res.title.clone(),
                url: res.url.clone(),
                source_type: res.source_type.clone(),
            });
        }

        for skill_out in executed_skills {
            combined_content.push_str(&skill_out);
            combined_content.push_str("\n\n");
            sources.push(SourceEntry {
                title: "Marketplace Skill Evidence".to_string(),
                url: "https://github.com/Panniantong/Agent-Reach".to_string(),
                source_type: "marketplace_skill".to_string(),
            });
        }

        if combined_content.is_empty() {
            combined_content = "Limited public search results available for this specific query angle.".to_string();
        }

        // 4. Synthesize notes using LLM worker prompt
        let prompt_text = self
            .prompts
            .render_worker(&sub_query.query, &sub_query.target_angle, &combined_content)
            .map_err(|e| AppError::Config(format!("Failed to render worker prompt: {}", e)))?;

        let messages = vec![ChatMessage::user(prompt_text)];
        let (findings, usage) = self.llm.complete_with_usage(&messages, Some(0.2)).await?;

        let note = ResearchNote {
            sub_query: sub_query.query.clone(),
            target_angle: sub_query.target_angle.clone(),
            findings,
            sources,
        };

        Ok((note, usage))
    }
}
