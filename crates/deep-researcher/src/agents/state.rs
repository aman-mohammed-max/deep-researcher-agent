use crate::llm::TokenUsage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubQuery {
    pub query: String,
    pub target_angle: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPlan {
    pub title: String,
    pub strategy: String,
    pub sub_queries: Vec<SubQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntry {
    pub title: String,
    pub url: String,
    pub source_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchNote {
    pub sub_query: String,
    pub target_angle: String,
    pub findings: String,
    pub sources: Vec<SourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchState {
    pub job_id: String,
    pub user_query: String,
    pub plan: Option<ResearchPlan>,
    pub research_notes: Vec<ResearchNote>,
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
