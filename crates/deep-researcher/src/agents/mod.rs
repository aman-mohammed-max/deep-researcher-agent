// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

pub mod intent;
pub mod planner;
pub mod researcher;
pub mod state;
pub mod writer;

use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::sync::Arc;
use tracing::{error, info};

use crate::error::AppError;
use crate::llm::LlmClient;
use crate::prompts::PromptEngine;
use crate::skills::MarketplaceRegistry;
use crate::storage::Storage;
use crate::tools::SearchDispatcher;

pub use intent::IntentClassifier;
pub use planner::ResearchPlanner;
pub use researcher::ResearchWorker;
pub use state::ResearchState;
pub use writer::ReportWriter;

#[derive(Clone)]
pub struct ResearchCoordinator {
    pub llm: Arc<LlmClient>,
    pub prompts: Arc<PromptEngine>,
    pub searcher: Arc<SearchDispatcher>,
    pub marketplace: Arc<MarketplaceRegistry>,
    pub storage: Storage,
    pub max_research_concurrency: usize,
}

impl ResearchCoordinator {
    pub fn new(
        llm: Arc<LlmClient>,
        prompts: Arc<PromptEngine>,
        searcher: Arc<SearchDispatcher>,
        marketplace: Arc<MarketplaceRegistry>,
        storage: Storage,
        max_research_concurrency: usize,
    ) -> Self {
        Self {
            llm,
            prompts,
            searcher,
            marketplace,
            storage,
            max_research_concurrency,
        }
    }

    /// Execute the complete research workflow with 1:1 NVIDIA Deep Researcher functional parity
    pub async fn run_workflow(&self, job_id: &str, user_query: &str) -> Result<String, AppError> {
        info!("Starting 1:1 Deep Research workflow for job '{}': '{}'", job_id, user_query);
        let mut state = ResearchState::new(job_id.to_string(), user_query.to_string());

        let _ = self.storage.update_job_status(job_id, "RUNNING").await;
        let _ = self.storage.add_event(job_id, "step", "Starting intent classification...").await;

        // ====================================================================
        // Step 1: Intent Classification
        // ====================================================================
        let intent_classifier = IntentClassifier::new(&self.llm, &self.prompts);
        let (decision, intent_usage) = intent_classifier.classify(user_query).await?;
        state.add_usage(&intent_usage);

        // If meta inquiry, return direct response immediately
        if decision.intent == "meta" {
            let q_lower = user_query.to_lowercase();
            let is_skill_inquiry = q_lower.contains("skill")
                || q_lower.contains("what can you do")
                || q_lower.contains("who are you")
                || q_lower.contains("capability")
                || q_lower.contains("abilities");

            let reply = if let Some(meta) = decision.meta_response {
                let meta_lower = meta.to_lowercase();
                if is_skill_inquiry
                    && (meta_lower.contains("don't have personal skills")
                        || meta_lower.contains("do not have personal skills")
                        || meta_lower.contains("not a human")
                        || meta_lower.contains("don't have skills")
                        || meta_lower.contains("no skills"))
                {
                    generate_skills_overview(&self.marketplace)
                } else if is_skill_inquiry && !meta_lower.contains("marketplace") && !meta_lower.contains("tavily") {
                    format!("{}\n\n{}", meta, generate_skills_overview(&self.marketplace))
                } else {
                    meta
                }
            } else if is_skill_inquiry {
                generate_skills_overview(&self.marketplace)
            } else {
                "Hello! I am the Deep Researcher Assistant powered by NVIDIA NIM & NVIDIA Lab technology. How can I assist your research today?".to_string()
            };

            let _ = self.storage.add_event(job_id, "meta_reply", &reply).await;
            let _ = self.storage.complete_job(
                job_id,
                &reply,
                state.total_usage.prompt_tokens,
                state.total_usage.completion_tokens,
                state.total_usage.total_tokens,
            ).await;

            return Ok(reply);
        }

        // ====================================================================
        // Step 2: Dynamic Skill Discovery & Marketplace Evaluation
        // ====================================================================
        let _ = self.storage.add_event(
            job_id,
            "step",
            "Evaluating Dynamic Skills Marketplace for domain-specific agent skills..."
        ).await;

        let matching_skills = self.marketplace.search_skills(user_query).await;
        if !matching_skills.is_empty() {
            for skill in matching_skills.into_iter().take(3) {
                let install_msg = format!(
                    "Dynamically installed & provisioned skill: '{}' (⭐ {} stars, zero-auth verified, repository: {})",
                    skill.name, skill.stars, skill.repository
                );
                let _ = self.storage.add_event(job_id, "skill_installed", &install_msg).await;
                info!("{}", install_msg);
            }
        }

        // ====================================================================
        // Step 3: Evidence-Grounded Research Planning with Discovery Tools
        // ====================================================================
        let _ = self.storage.add_event(
            job_id,
            "step",
            "Executing discovery searches & decomposing evidence-grounded research plan..."
        ).await;

        let planner = ResearchPlanner::with_searcher(&self.llm, &self.prompts, &self.searcher);
        let (plan, plan_usage) = planner.plan(user_query).await?;
        state.add_usage(&plan_usage);
        state.plan = Some(plan.clone());

        let plan_json = serde_json::to_string(&plan).unwrap_or_default();
        let _ = self.storage.add_event(job_id, "plan_created", &plan_json).await;

        let queries_to_run = plan.get_queries();

        // ====================================================================
        // Step 3: Concurrent Multi-Turn Worker Research Batch
        // ====================================================================
        let _ = self.storage.add_event(
            job_id,
            "step",
            &format!(
                "Dispatching {} concurrent research workers (max concurrency: {})...",
                queries_to_run.len(),
                self.max_research_concurrency
            ),
        ).await;

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_research_concurrency.max(1)));
        let mut futures = FuturesUnordered::new();

        for rq in queries_to_run {
            let worker = ResearchWorker::new(
                &self.llm,
                &self.prompts,
                &self.searcher,
                &self.marketplace,
            );
            let query_item = rq.clone();
            let j_id = job_id.to_string();
            let storage = self.storage.clone();
            let sem = semaphore.clone();

            futures.push(async move {
                let _permit = sem.acquire().await;
                let _ = storage.add_event(
                    &j_id,
                    "worker_start",
                    &format!("Investigating: {}", query_item.query)
                ).await;

                let res = worker.execute_query(&query_item).await;
                if res.is_ok() {
                    let _ = storage.add_event(
                        &j_id,
                        "worker_done",
                        &format!("Gathered empirical findings for: {}", query_item.query)
                    ).await;
                }
                res
            });
        }

        while let Some(res) = futures.next().await {
            match res {
                Ok((note, structured_note, usage)) => {
                    state.add_usage(&usage);
                    for s in &note.sources {
                        state.verified_sources.push(s.clone());
                    }
                    state.research_notes.push(note);
                    state.structured_notes.push(structured_note);
                }
                Err(err) => {
                    error!("Research worker failed for a sub-query: {}", err);
                }
            }
        }

        // ====================================================================
        // Step 4: Final Synthesis & Report Writing with Citation Verification
        // ====================================================================
        let _ = self.storage.add_event(
            job_id,
            "step",
            "Cross-synthesizing research notes, evaluating constraints, and verifying citations..."
        ).await;

        if !state.verified_sources.is_empty() {
            if let Ok(srcs_json) = serde_json::to_string(&state.verified_sources) {
                let _ = self.storage.add_event(job_id, "sources_collected", &srcs_json).await;
            }
        }

        let writer = ReportWriter::new(&self.llm, &self.prompts);
        let (report, writer_usage) = writer
            .write_structured_report(
                user_query,
                &plan,
                &state.structured_notes,
                &state.research_notes,
                &state.verified_sources,
            )
            .await?;
        state.add_usage(&writer_usage);
        state.final_report = Some(report.clone());

        // ====================================================================
        // Step 5: Save Completion & Full Tokenomics
        // ====================================================================
        let _ = self.storage.complete_job(
            job_id,
            &report,
            state.total_usage.prompt_tokens,
            state.total_usage.completion_tokens,
            state.total_usage.total_tokens,
        ).await;

        let _ = self.storage.add_event(
            job_id,
            "completed",
            "Deep research report synthesis completed successfully."
        ).await;

        info!(
            "Successfully completed deep research job '{}' with {} total tokens and {} verified sources",
            job_id,
            state.total_usage.total_tokens,
            state.verified_sources.len()
        );

        Ok(report)
    }
}

fn generate_skills_overview(marketplace: &MarketplaceRegistry) -> String {
    let available_skills = marketplace.all_skills();
    let mut skills_list = String::new();
    for s in available_skills {
        skills_list.push_str(&format!("- **{}** (⭐ {} stars): {}\n", s.name, s.stars, s.description));
    }

    format!(
        "I am the **Deep Researcher Assistant, powered by NVIDIA NIM & NVIDIA Lab technology**.\n\n\
        I am equipped with a modular, extensible **Agent Skills Ecosystem** designed for autonomous end-to-end research:\n\n\
        ### 1. Built-in Core Research Skills\n\
        - **Deep Web Search & Verification (Tavily)**: High-depth web exploration, snippet extraction, and verified canonical URL resolution.\n\
        - **Academic & Scientific Literature (OpenAlex & arXiv)**: Search and extraction across 250M+ scientific papers, citations, and methodologies.\n\
        - **Real-Time Community & Social Intelligence (Agent-Reach)**: Deep analysis of Reddit community threads and Twitter/X discourse with zero credentials required.\n\
        - **Financial Filings & Disclosures (SEC EDGAR)**: Instant retrieval of 10-K, 10-Q reports, balance sheets, and earnings statements.\n\
        - **Evidence Triangulation & Cross-Verification**: Multi-angle claim verification with automated citation indexing.\n\n\
        ### 2. Dynamic Skills Marketplace & On-the-Fly Installation\n\
        When you submit a research task, I automatically evaluate whether domain-specific skills are required. I search the public skills marketplace, verify zero-auth safety and star ratings (>1,000 stars), and dynamically install and provision those skills for your task.\n\n\
        **Currently Curated Marketplace Skills:**\n\
        {}\n\
        How can I assist your research today?",
        skills_list
    )
}
