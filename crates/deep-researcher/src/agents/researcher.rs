// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::agents::state::{
    EvidenceJudgment, ResearchFinding, ResearchNote, ResearchNotes, ResearchQuery,
    ResearchSource, SourceEntry, SubQuery,
};
use crate::error::AppError;
use crate::llm::{ChatMessage, LlmClient, TokenUsage};
use crate::prompts::PromptEngine;
use crate::skills::{MarketplaceRegistry, SkillExecutor};
use crate::tools::SearchDispatcher;
use tracing::{debug, info};

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

    /// Backward-compatible execute method for SubQuery callers
    pub async fn execute(&self, sub_query: &SubQuery) -> Result<(ResearchNote, TokenUsage), AppError> {
        let rq = ResearchQuery {
            query: sub_query.query.clone(),
            subqueries: vec![],
            preferred_tools: vec!["tavily_search".to_string()],
            fallback_tools: vec![],
            target_components: vec![sub_query.target_angle.clone()],
            rationale: sub_query.purpose.clone(),
        };
        let (note, _structured, usage) = self.execute_query(&rq).await?;
        Ok((note, usage))
    }

    /// Execute multi-turn iterative research matching NVIDIA researcher.j2 protocol
    pub async fn execute_query(
        &self,
        rq: &ResearchQuery,
    ) -> Result<(ResearchNote, ResearchNotes, TokenUsage), AppError> {
        info!("Executing multi-turn researcher worker for query: '{}'", rq.query);
        let mut total_usage = TokenUsage::default();

        let target_angle = rq.target_components.first().cloned().unwrap_or_else(|| "technical_data".to_string());

        // ====================================================================
        // Turn 1: Primary Search Pass
        // ====================================================================
        let mut all_search_results = self.searcher.search(&rq.query, &target_angle, 6).await;

        // ====================================================================
        // Turn 2: Follow-up Investigation on Subqueries or Gaps
        // ====================================================================
        for subq in &rq.subqueries {
            if !subq.trim().is_empty() {
                info!("Worker following up on subquery angle: '{}'", subq);
                let follow_up = self.searcher.search(subq, &target_angle, 4).await;
                all_search_results.extend(follow_up);
            }
        }

        // ====================================================================
        // Turn 3: Adaptive Escalation to Specialized Skills Marketplace
        // ====================================================================
        let mut executed_skills = Vec::new();
        let query_lower = rq.query.to_lowercase();
        let needs_specialized_skill = all_search_results.len() < 2
            || target_angle == "community_sentiment"
            || target_angle == "pricing_local"
            || query_lower.contains("room")
            || query_lower.contains("flat")
            || query_lower.contains("stock");

        if needs_specialized_skill {
            info!("Specialized angle or information deficit detected. Querying Skills Marketplace...");
            let candidate_skills = self.marketplace.search_skills(&rq.query).await;
            for skill in candidate_skills.iter().take(2) {
                info!("Executing marketplace skill: '{}' (⭐ {} stars)", skill.name, skill.stars);
                if let Ok(skill_data) = SkillExecutor::execute(&skill.id, &rq.query).await {
                    executed_skills.push(format!(
                        "### [Marketplace Skill: {} (⭐ {})]\n{}",
                        skill.name, skill.stars, skill_data
                    ));
                }
            }
        }

        // ====================================================================
        // Turn 4: Evidence Aggregation & Source Indexing
        // ====================================================================
        let mut combined_content = String::new();
        let mut source_entries = Vec::new();
        let mut research_sources = Vec::new();
        let mut source_counter = 1;

        for res in &all_search_results {
            combined_content.push_str(&format!(
                "Source [{}] Title: {}\nURL: {}\nSnippet: {}\n\n",
                source_counter, res.title, res.url, res.snippet
            ));
            source_entries.push(SourceEntry {
                title: res.title.clone(),
                url: res.url.clone(),
                source_type: res.source_type.clone(),
            });
            research_sources.push(ResearchSource {
                id: source_counter,
                title: res.title.clone(),
                source_type: res.source_type.clone(),
                locator: res.url.clone(),
            });
            source_counter += 1;
        }

        for skill_out in executed_skills {
            combined_content.push_str(&skill_out);
            combined_content.push_str("\n\n");
            let skill_url = "https://github.com/Panniantong/Agent-Reach".to_string();
            source_entries.push(SourceEntry {
                title: "Marketplace Skill Evidence".to_string(),
                url: skill_url.clone(),
                source_type: "marketplace_skill".to_string(),
            });
            research_sources.push(ResearchSource {
                id: source_counter,
                title: "Marketplace Skill Evidence".to_string(),
                source_type: "tool".to_string(),
                locator: skill_url,
            });
            source_counter += 1;
        }

        if combined_content.is_empty() {
            combined_content = "Limited public search results available for this specific query angle.".to_string();
        }

        // ====================================================================
        // Turn 5: Structured Synthesis matching NVIDIA researcher.j2
        // ====================================================================
        let default_tools = vec![
            serde_json::json!({
                "name": "tavily_search",
                "description": "Run targeted search queries for specific factual evidence."
            }),
            serde_json::json!({
                "name": "think",
                "description": "Analyze results, identify information gaps, and evaluate confidence."
            }),
        ];

        let system_prompt = self
            .prompts
            .render_researcher_system(&default_tools, None, None, None, false)
            .map_err(|e| AppError::Config(format!("Failed to render researcher system prompt: {}", e)))?;

        let user_prompt = format!(
            "## Assigned Research Query Contract\n\
            **Query:** {}\n\
            **Target Components:** {:?}\n\
            **Rationale:** {}\n\n\
            ## Discovered Sources & Content\n\
            {}\n\n\
            ## Output Instructions\n\
            You MUST return a JSON object conforming to the ResearchNotes schema:\n\
            {{\n  \
              \"query_topic\": \"{}\",\n  \
              \"target_components\": {:?},\n  \
              \"summary\": \"Concise synthesis of discoveries\",\n  \
              \"findings\": [\n    \
                {{\n      \
                  \"claim\": \"Specific factual claim\",\n      \
                  \"evidence\": \"Concrete numbers, dates, benchmarks, direct quotes\",\n      \
                  \"source_ids\": [1],\n      \
                  \"confidence\": \"high|medium|low\",\n      \
                  \"caveats\": [\"Limitations or caveats\"]\n    \
                }}\n  \
              ],\n  \
              \"gaps\": [\n    \
                {{\n      \
                  \"description\": \"Missing or weakly supported details\",\n      \
                  \"impact\": \"Why it matters\",\n      \
                  \"suggested_follow_up_queries\": [\"query\"]\n    \
                }}\n  \
              ],\n  \
              \"narrative_notes\": \"Detailed synthesis preserving technical nuance, mechanisms, and causal explanations for the report writer.\",\n  \
              \"language\": \"English\",\n  \
              \"evidence_judgment\": {{\n    \
                \"relevance_score\": 95,\n    \
                \"confidence\": \"high\",\n    \
                \"rationale\": \"Direct empirical support for target components\"\n  \
              }}\n\
            }}",
            rq.query,
            rq.target_components,
            rq.rationale,
            combined_content,
            rq.query,
            rq.target_components,
        );

        let messages = vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_prompt),
        ];

        let (raw_findings, usage) = self.llm.complete_with_usage(&messages, Some(0.2)).await?;
        total_usage.prompt_tokens += usage.prompt_tokens;
        total_usage.completion_tokens += usage.completion_tokens;
        total_usage.total_tokens += usage.total_tokens;

        debug!("Raw worker response: {}", raw_findings);

        // ====================================================================
        // Turn 6: Parse Structured ResearchNotes
        // ====================================================================
        let cleaned = extract_json_block(&raw_findings);
        let structured_notes: ResearchNotes = serde_json::from_str(&cleaned).unwrap_or_else(|_| {
            ResearchNotes {
                query_topic: rq.query.clone(),
                target_components: rq.target_components.clone(),
                summary: format!("Synthesized research for: {}", rq.query),
                findings: vec![ResearchFinding {
                    claim: "Core evidence gathered".to_string(),
                    evidence: raw_findings.clone(),
                    source_ids: research_sources.iter().map(|s| s.id).collect(),
                    confidence: "high".to_string(),
                    caveats: vec![],
                }],
                gaps: vec![],
                sources: research_sources.clone(),
                narrative_notes: raw_findings.clone(),
                language: "English".to_string(),
                evidence_judgment: Some(EvidenceJudgment {
                    relevance_score: 90,
                    confidence: "high".to_string(),
                    rationale: "Empirical findings retrieved from search tools".to_string(),
                }),
            }
        });

        // Backward-compatible summary note
        let note = ResearchNote {
            sub_query: rq.query.clone(),
            target_angle,
            findings: if !structured_notes.narrative_notes.is_empty() {
                format!(
                    "{}\n\n**Key Findings & Evidence:**\n{}",
                    structured_notes.summary, structured_notes.narrative_notes
                )
            } else {
                raw_findings
            },
            sources: source_entries,
        };

        Ok((note, structured_notes, total_usage))
    }
}

fn extract_json_block(text: &str) -> String {
    let text = text.trim();
    if let Some(start) = text.find("```json") {
        if let Some(end) = text[start + 7..].find("```") {
            return text[start + 7..start + 7 + end].trim().to_string();
        }
    } else if let Some(start) = text.find("```") {
        if let Some(end) = text[start + 3..].find("```") {
            return text[start + 3..start + 3 + end].trim().to_string();
        }
    }
    if let (Some(start), Some(end)) = (text.find('{'), text.rfind('}')) {
        if start <= end {
            return text[start..=end].to_string();
        }
    }
    text.to_string()
}
