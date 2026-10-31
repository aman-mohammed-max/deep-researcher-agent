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

    /// Execute the complete research workflow end-to-end
    pub async fn run_workflow(&self, job_id: &str, user_query: &str) -> Result<String, AppError> {
        info!("Starting research workflow for job '{}': '{}'", job_id, user_query);
        let mut state = ResearchState::new(job_id.to_string(), user_query.to_string());

        let _ = self.storage.update_job_status(job_id, "RUNNING").await;
        let _ = self.storage.add_event(job_id, "step", "Starting intent classification...").await;

        // Step 1: Intent Classification
        let intent_classifier = IntentClassifier::new(&self.llm, &self.prompts);
        let (decision, intent_usage) = intent_classifier.classify(user_query).await?;
        state.add_usage(&intent_usage);

        // If meta inquiry, return direct response immediately
        if decision.intent == "meta" {
            let reply = decision.meta_response.unwrap_or_else(|| {
                "Hello! I am your AI Deep Research Assistant. How can I assist your research today?".to_string()
            });

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

        let _ = self.storage.add_event(job_id, "step", "Generating research plan and decomposing queries...").await;

        // Step 2: Research Planning
        let planner = ResearchPlanner::new(&self.llm, &self.prompts);
        let (plan, plan_usage) = planner.plan(user_query).await?;
        state.add_usage(&plan_usage);
        state.plan = Some(plan.clone());

        let plan_json = serde_json::to_string(&plan).unwrap_or_default();
        let _ = self.storage.add_event(job_id, "plan_created", &plan_json).await;

        // Step 3: Concurrent Worker Research Batch
        let _ = self.storage.add_event(
            job_id,
            "step",
            &format!("Dispatching {} concurrent research workers (max concurrency: {})...", plan.sub_queries.len(), self.max_research_concurrency),
        ).await;

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_research_concurrency.max(1)));
        let mut futures = FuturesUnordered::new();
        for sub_query in &plan.sub_queries {
            let worker = ResearchWorker::new(
                &self.llm,
                &self.prompts,
                &self.searcher,
                &self.marketplace,
            );
            let sq = sub_query.clone();
            let j_id = job_id.to_string();
            let storage = self.storage.clone();
            let sem = semaphore.clone();

            futures.push(async move {
                let _permit = sem.acquire().await;
                let _ = storage.add_event(&j_id, "worker_start", &format!("Investigating: {}", sq.query)).await;
                let res = worker.execute(&sq).await;
                if res.is_ok() {
                    let _ = storage.add_event(&j_id, "worker_done", &format!("Gathered findings for: {}", sq.query)).await;
                }
                res
            });
        }

        while let Some(res) = futures.next().await {
            match res {
                Ok((note, usage)) => {
                    state.add_usage(&usage);
                    for s in &note.sources {
                        state.verified_sources.push(s.clone());
                    }
                    state.research_notes.push(note);
                }
                Err(err) => {
                    error!("Research worker failed for a sub-query: {}", err);
                }
            }
        }

        // Step 4: Final Synthesis & Report Writing
        let _ = self.storage.add_event(job_id, "step", "Synthesizing research notes and verifying citations...").await;

        let writer = ReportWriter::new(&self.llm, &self.prompts);
        let (report, writer_usage) = writer
            .write_report(
                user_query,
                &plan.title,
                &state.research_notes,
                &state.verified_sources,
            )
            .await?;
        state.add_usage(&writer_usage);
        state.final_report = Some(report.clone());

        // Step 5: Save Completion & Tokenomics
        let _ = self.storage.complete_job(
            job_id,
            &report,
            state.total_usage.prompt_tokens,
            state.total_usage.completion_tokens,
            state.total_usage.total_tokens,
        ).await;

        let _ = self.storage.add_event(job_id, "completed", "Research report synthesis completed successfully.").await;
        info!("Successfully completed research job '{}' with {} total tokens", job_id, state.total_usage.total_tokens);

        Ok(report)
    }
}
