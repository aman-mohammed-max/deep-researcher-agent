use crate::error::AppError;
use crate::llm::{ChatMessage, LlmClient, TokenUsage};
use crate::prompts::PromptEngine;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentDecision {
    pub intent: String,
    pub meta_response: Option<String>,
    pub research_depth: Option<String>,
    pub reasoning: Option<String>,
}

pub struct IntentClassifier<'a> {
    llm: &'a LlmClient,
    prompts: &'a PromptEngine,
}

impl<'a> IntentClassifier<'a> {
    pub fn new(llm: &'a LlmClient, prompts: &'a PromptEngine) -> Self {
        Self { llm, prompts }
    }

    pub async fn classify(&self, user_query: &str) -> Result<(IntentDecision, TokenUsage), AppError> {
        let prompt_text = self
            .prompts
            .render_intent(user_query)
            .map_err(|e| AppError::Config(format!("Failed to render intent prompt: {}", e)))?;

        let messages = vec![ChatMessage::user(prompt_text)];
        let (raw_response, usage) = self.llm.complete_with_usage(&messages, Some(0.1)).await?;

        debug!("Raw intent response: {}", raw_response);

        // Extract JSON from response
        let cleaned = extract_json_block(&raw_response);
        let decision: IntentDecision = serde_json::from_str(&cleaned).unwrap_or_else(|_| {
            let q_lower = user_query.to_lowercase();
            if q_lower.contains("skill")
                || q_lower.contains("who are you")
                || q_lower.contains("what can you do")
                || q_lower.contains("capability")
                || q_lower.contains("abilities")
                || q_lower.contains("hello")
                || q_lower.contains("hi")
            {
                IntentDecision {
                    intent: "meta".to_string(),
                    meta_response: None,
                    research_depth: None,
                    reasoning: Some("Meta capability/identity inquiry".to_string()),
                }
            } else {
                IntentDecision {
                    intent: "research".to_string(),
                    meta_response: None,
                    research_depth: Some("deep".to_string()),
                    reasoning: Some("Defaulting to deep research".to_string()),
                }
            }
        });

        info!(
            "Classified intent: '{}' (depth: '{:?}')",
            decision.intent, decision.research_depth
        );

        Ok((decision, usage))
    }
}

fn extract_json_block(text: &str) -> String {
    let text = text.trim();
    if let (Some(start), Some(end)) = (text.find('{'), text.rfind('}')) {
        if start <= end {
            return text[start..=end].to_string();
        }
    }
    text.to_string()
}
