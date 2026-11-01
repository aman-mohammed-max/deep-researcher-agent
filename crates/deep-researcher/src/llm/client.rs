use crate::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[allow(dead_code)]
impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
struct CompletionPayload<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct CompletionResponse {
    choices: Vec<Choice>,
    usage: Option<TokenUsage>,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    #[serde(default)]
    #[allow(dead_code)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    pub base_url: String,
    pub api_key: String,
    pub default_model: String,
}

impl LlmClient {
    pub fn new(base_url: String, api_key: String, default_model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            default_model,
        }
    }

    #[allow(dead_code)]
    pub async fn complete(&self, messages: &[ChatMessage]) -> Result<String, AppError> {
        let (content, _) = self.complete_with_usage(messages, None).await?;
        Ok(content)
    }

    pub async fn complete_with_usage(
        &self,
        messages: &[ChatMessage],
        temperature: Option<f32>,
    ) -> Result<(String, TokenUsage), AppError> {
        self.complete_with_model(messages, &self.default_model.clone(), temperature).await
    }

    pub async fn complete_with_model(
        &self,
        messages: &[ChatMessage],
        model: &str,
        temperature: Option<f32>,
    ) -> Result<(String, TokenUsage), AppError> {
        let endpoint = format!("{}/chat/completions", self.base_url);

        let is_nim = model.contains("nemotron") || self.base_url.contains("nvidia");
        let chat_template_kwargs = if is_nim {
            Some(serde_json::json!({ "enable_thinking": false }))
        } else {
            None
        };

        let payload = CompletionPayload {
            model,
            messages,
            temperature: temperature.unwrap_or(0.2),
            max_tokens: 8192,
            chat_template_kwargs,
        };

        debug!("Calling LLM endpoint: {} with model: {}", endpoint, model);

        let mut req = self.client.post(&endpoint).json(&payload);

        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        let resp = req.send().await.map_err(|e| {
            error!("LLM request failed: {}", e);
            AppError::Network(format!("Failed to connect to LLM provider: {}", e))
        })?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            error!("LLM provider returned error {}: {}", status, err_text);
            return Err(AppError::Llm(format!(
                "LLM provider error ({}): {}",
                status, err_text
            )));
        }

        let parsed: CompletionResponse = resp.json().await.map_err(|e| {
            error!("Failed to parse LLM JSON response: {}", e);
            AppError::Llm(format!("Invalid response from LLM provider: {}", e))
        })?;

        let text = parsed
            .choices
            .first()
            .and_then(|c| {
                if let Some(ref cnt) = c.message.content {
                    if !cnt.trim().is_empty() {
                        return Some(cnt.clone());
                    }
                }
                c.message.reasoning_content.clone()
            })
            .unwrap_or_default();

        let usage = parsed.usage.unwrap_or_default();

        info!(
            "LLM completion succeeded (model: {}, tokens: {} [in: {}, out: {}])",
            model, usage.total_tokens, usage.prompt_tokens, usage.completion_tokens
        );

        Ok((text, usage))
    }
}
