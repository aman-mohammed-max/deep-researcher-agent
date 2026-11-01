// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::llm::TokenUsage;
use serde::{Deserialize, Serialize};

// ============================================================================
// 1. Structured Planner Contracts (Matching subagent_contracts.py)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskAnalysis {
    #[serde(default)]
    pub user_intent: String,
    #[serde(default)]
    pub explicit_requirements: Vec<String>,
    #[serde(default)]
    pub implicit_requirements: Vec<String>,
    #[serde(default)]
    pub out_of_scope: Vec<String>,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "English".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnswerComponent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum HelperComponent {
    Structured(AnswerComponent),
    Simple(String),
}

fn deserialize_components<'de, D>(deserializer: D) -> Result<Vec<AnswerComponent>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<Vec<HelperComponent>> = Option::deserialize(deserializer)?;
    let helpers = opt.unwrap_or_default();
    Ok(helpers
        .into_iter()
        .map(|h| match h {
            HelperComponent::Structured(c) => c,
            HelperComponent::Simple(s) => AnswerComponent {
                id: s.clone(),
                name: s.clone(),
                description: s,
            },
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnswerStrategy {
    #[serde(default = "default_answer_type")]
    pub answer_type: String, // "long_form_report", "brief_answer", "table", "comparison", etc.
    #[serde(default)]
    pub title: String,
    #[serde(default, deserialize_with = "deserialize_components")]
    pub required_components: Vec<AnswerComponent>,
}

fn default_answer_type() -> String {
    "long_form_report".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Constraint {
    #[serde(default)]
    pub category: String, // "content", "source", "structure", "depth", "format", "exclusion"
    #[serde(default)]
    pub constraint: String,
    #[serde(default)]
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum HelperConstraint {
    Structured(Constraint),
    Simple(String),
}

fn deserialize_constraints<'de, D>(deserializer: D) -> Result<Vec<Constraint>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<Vec<HelperConstraint>> = Option::deserialize(deserializer)?;
    let helpers = opt.unwrap_or_default();
    Ok(helpers
        .into_iter()
        .map(|h| match h {
            HelperConstraint::Structured(c) => c,
            HelperConstraint::Simple(s) => Constraint {
                category: "general".to_string(),
                constraint: s.clone(),
                rationale: s,
            },
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchQuery {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub subqueries: Vec<String>,
    #[serde(default = "default_preferred_tools")]
    pub preferred_tools: Vec<String>,
    #[serde(default)]
    pub fallback_tools: Vec<String>,
    #[serde(default)]
    pub target_components: Vec<String>,
    #[serde(default)]
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum HelperQuery {
    Structured(ResearchQuery),
    Simple(String),
}

fn deserialize_queries<'de, D>(deserializer: D) -> Result<Vec<ResearchQuery>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<Vec<HelperQuery>> = Option::deserialize(deserializer)?;
    let helpers = opt.unwrap_or_default();
    Ok(helpers
        .into_iter()
        .map(|h| match h {
            HelperQuery::Structured(q) => q,
            HelperQuery::Simple(s) => ResearchQuery {
                query: s.clone(),
                subqueries: vec![],
                preferred_tools: vec!["tavily_search".to_string()],
                fallback_tools: vec![],
                target_components: vec![],
                rationale: s,
            },
        })
        .collect())
}

fn default_preferred_tools() -> Vec<String> {
    vec!["tavily_search".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubQuery {
    pub query: String,
    #[serde(default)]
    pub target_angle: String,
    #[serde(default)]
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchPlan {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub strategy: String,
    #[serde(default)]
    pub task_analysis: TaskAnalysis,
    #[serde(default)]
    pub answer_strategy: AnswerStrategy,
    #[serde(default, deserialize_with = "deserialize_constraints")]
    pub constraints: Vec<Constraint>,
    #[serde(default, deserialize_with = "deserialize_queries")]
    pub queries: Vec<ResearchQuery>,
    #[serde(default)]
    pub sub_queries: Vec<SubQuery>,
}

impl ResearchPlan {
    /// Return the canonical list of queries to execute across workers
    pub fn get_queries(&self) -> Vec<ResearchQuery> {
        if !self.queries.is_empty() {
            self.queries.clone()
        } else if !self.sub_queries.is_empty() {
            self.sub_queries
                .iter()
                .map(|sq| ResearchQuery {
                    query: sq.query.clone(),
                    subqueries: vec![],
                    preferred_tools: vec!["tavily_search".to_string()],
                    fallback_tools: vec![],
                    target_components: vec![sq.target_angle.clone()],
                    rationale: sq.purpose.clone(),
                })
                .collect()
        } else {
            vec![ResearchQuery {
                query: self.title.clone(),
                subqueries: vec![],
                preferred_tools: vec!["tavily_search".to_string()],
                fallback_tools: vec![],
                target_components: vec![],
                rationale: "Default core research".to_string(),
            }]
        }
    }
}

// ============================================================================
// 2. Structured Evidence & Researcher Contracts
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceEntry {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_source_type")]
    pub source_type: String,
}

fn default_source_type() -> String {
    "url".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchSource {
    #[serde(default)]
    pub id: usize,
    #[serde(default)]
    pub title: String,
    #[serde(default = "default_source_type")]
    pub source_type: String, // "url", "internal_document", "tool"
    #[serde(default)]
    pub locator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchFinding {
    #[serde(default)]
    pub claim: String,
    #[serde(default)]
    pub evidence: String,
    #[serde(default)]
    pub source_ids: Vec<usize>,
    #[serde(default = "default_confidence")]
    pub confidence: String, // "low", "medium", "high"
    #[serde(default)]
    pub caveats: Vec<String>,
}

fn default_confidence() -> String {
    "high".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchGap {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub suggested_follow_up_queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvidenceJudgment {
    #[serde(default = "default_relevance_score")]
    pub relevance_score: u32, // 0 to 100
    #[serde(default = "default_confidence")]
    pub confidence: String,
    #[serde(default)]
    pub rationale: String,
}

fn default_relevance_score() -> u32 {
    90
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchNotes {
    #[serde(default)]
    pub query_topic: String,
    #[serde(default)]
    pub target_components: Vec<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub findings: Vec<ResearchFinding>,
    #[serde(default)]
    pub gaps: Vec<ResearchGap>,
    #[serde(default)]
    pub sources: Vec<ResearchSource>,
    #[serde(default)]
    pub narrative_notes: String,
    #[serde(default = "default_language")]
    pub language: String,
    pub evidence_judgment: Option<EvidenceJudgment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchNote {
    pub sub_query: String,
    pub target_angle: String,
    pub findings: String,
    pub sources: Vec<SourceEntry>,
}

// ============================================================================
// 3. Coordinator State Management
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchState {
    pub job_id: String,
    pub user_query: String,
    pub plan: Option<ResearchPlan>,
    pub research_notes: Vec<ResearchNote>,
    pub structured_notes: Vec<ResearchNotes>,
    pub verified_sources: Vec<SourceEntry>,
    pub final_report: Option<String>,
    pub total_usage: TokenUsage,
}

impl ResearchState {
    pub fn new(job_id: String, user_query: String) -> Self {
        Self {
            job_id,
            user_query,
            plan: None,
            research_notes: Vec::new(),
            structured_notes: Vec::new(),
            verified_sources: Vec::new(),
            final_report: None,
            total_usage: TokenUsage::default(),
        }
    }

    pub fn add_usage(&mut self, usage: &TokenUsage) {
        self.total_usage.prompt_tokens += usage.prompt_tokens;
        self.total_usage.completion_tokens += usage.completion_tokens;
        self.total_usage.total_tokens += usage.total_tokens;
    }
}
