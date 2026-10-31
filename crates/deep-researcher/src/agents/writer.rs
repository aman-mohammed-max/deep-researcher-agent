use crate::agents::state::{ResearchNote, SourceEntry};
use crate::error::AppError;
use crate::llm::{ChatMessage, LlmClient, TokenUsage};
use crate::prompts::PromptEngine;
use tracing::info;

pub struct ReportWriter<'a> {
    llm: &'a LlmClient,
    prompts: &'a PromptEngine,
}

impl<'a> ReportWriter<'a> {
    pub fn new(llm: &'a LlmClient, prompts: &'a PromptEngine) -> Self {
        Self { llm, prompts }
    }

    pub async fn write_report(
        &self,
        user_query: &str,
        plan_title: &str,
        notes: &[ResearchNote],
        sources: &[SourceEntry],
    ) -> Result<(String, TokenUsage), AppError> {
        info!("Synthesizing final research report for query: '{}'", user_query);

        let mut compiled_notes = String::new();
        for (i, note) in notes.iter().enumerate() {
            compiled_notes.push_str(&format!(
                "### Section {}: {}\n**Target Angle**: {}\n**Findings & Evidence**:\n{}\n\n",
                i + 1,
                note.sub_query,
                note.target_angle,
                note.findings
            ));
        }

        let verified_sources_json = serde_json::to_value(sources).unwrap_or_default();

        let prompt_text = self
            .prompts
            .render_writer(user_query, plan_title, &compiled_notes, &verified_sources_json)
            .map_err(|e| AppError::Config(format!("Failed to render writer prompt: {}", e)))?;

        let messages = vec![ChatMessage::user(prompt_text)];
        let (report_markdown, usage) = self.llm.complete_with_usage(&messages, Some(0.3)).await?;

        // Citation verification check: ensure URLs from sources appear in report
        let verified_report = self.verify_and_append_sources(report_markdown, sources);

        Ok((verified_report, usage))
    }

    fn verify_and_append_sources(&self, mut report: String, sources: &[SourceEntry]) -> String {
        // Append an authoritative Sources & References section if not already present
        if !report.contains("## Sources") && !report.contains("## References") && !sources.is_empty() {
            report.push_str("\n\n## Sources & Citations\n\n");
            let mut seen_urls = std::collections::HashSet::new();

            for s in sources {
                if !s.url.is_empty() && seen_urls.insert(&s.url) {
                    report.push_str(&format!("- [{}]({})\n", s.title, s.url));
                }
            }
        }
        report
    }
}
