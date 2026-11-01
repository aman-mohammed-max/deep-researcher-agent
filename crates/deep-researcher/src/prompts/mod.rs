// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use minijinja::{context, Environment};
use serde_json::Value;

// Canonical NVIDIA Jinja2 Prompt Templates (100% Unabridged Byte-for-Byte Preservation)
pub const ORCHESTRATOR_TEMPLATE: &str = include_str!("templates/orchestrator.j2");
pub const PLANNER_TEMPLATE: &str = include_str!("templates/planner.j2");
pub const RESEARCHER_TEMPLATE: &str = include_str!("templates/researcher.j2");
pub const WRITER_TEMPLATE: &str = include_str!("templates/writer.j2");
pub const SOURCE_ROUTER_TEMPLATE: &str = include_str!("templates/source_router.j2");
pub const SOURCE_REGISTRY_TEMPLATE: &str = include_str!("templates/source_registry.j2");
pub const INTENT_TEMPLATE: &str = include_str!("templates/intent_classification.j2");
pub const CONTEXT_INTENT_TEMPLATE: &str = include_str!("templates/context_aware_intent_router.j2");

pub struct PromptEngine {
    env: Environment<'static>,
}

impl Default for PromptEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptEngine {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.add_template("orchestrator", ORCHESTRATOR_TEMPLATE).unwrap();
        env.add_template("planner", PLANNER_TEMPLATE).unwrap();
        env.add_template("researcher", RESEARCHER_TEMPLATE).unwrap();
        env.add_template("writer", WRITER_TEMPLATE).unwrap();
        env.add_template("source_router", SOURCE_ROUTER_TEMPLATE).unwrap();
        env.add_template("source_registry", SOURCE_REGISTRY_TEMPLATE).unwrap();
        env.add_template("intent", INTENT_TEMPLATE).unwrap();
        env.add_template("context_intent", CONTEXT_INTENT_TEMPLATE).unwrap();
        Self { env }
    }

    /// Render Intent Classifier system prompt
    pub fn render_intent_system(
        &self,
        current_datetime: Option<&str>,
        available_documents: Option<&Value>,
    ) -> Result<String, String> {
        let tmpl = self.env.get_template("intent").map_err(|e| e.to_string())?;
        let now_str = current_datetime
            .map(|s| s.to_string())
            .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        let docs = available_documents.cloned().unwrap_or_else(|| serde_json::json!([]));
        tmpl.render(context! {
            current_datetime => now_str,
            available_documents => docs,
        })
        .map_err(|e| e.to_string())
    }

    /// Render Research Planner system prompt matching NVIDIA factory.py
    pub fn render_planner_system(
        &self,
        tools: &[serde_json::Value],
        enable_source_router: bool,
        max_research_concurrency: usize,
        current_datetime: Option<&str>,
        user_info: Option<&Value>,
        available_documents: Option<&Value>,
    ) -> Result<String, String> {
        let tmpl = self.env.get_template("planner").map_err(|e| e.to_string())?;
        let now_str = current_datetime
            .map(|s| s.to_string())
            .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        let docs = available_documents.cloned().unwrap_or_else(|| serde_json::json!([]));
        let uinfo = user_info.cloned().unwrap_or_else(|| serde_json::json!({}));
        tmpl.render(context! {
            current_datetime => now_str,
            user_info => uinfo,
            available_documents => docs,
            tools => tools,
            enable_source_router => enable_source_router,
            max_research_concurrency => max_research_concurrency,
        })
        .map_err(|e| e.to_string())
    }

    /// Render Researcher Worker system prompt matching NVIDIA factory.py
    pub fn render_researcher_system(
        &self,
        tools: &[serde_json::Value],
        current_datetime: Option<&str>,
        user_info: Option<&Value>,
        available_documents: Option<&Value>,
        execution_enabled: bool,
    ) -> Result<String, String> {
        let tmpl = self.env.get_template("researcher").map_err(|e| e.to_string())?;
        let now_str = current_datetime
            .map(|s| s.to_string())
            .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        let docs = available_documents.cloned().unwrap_or_else(|| serde_json::json!([]));
        let uinfo = user_info.cloned().unwrap_or_else(|| serde_json::json!({}));
        tmpl.render(context! {
            current_datetime => now_str,
            user_info => uinfo,
            available_documents => docs,
            tools => tools,
            execution_enabled => execution_enabled,
            sandbox_workdir => "/tmp/workspace",
            sandbox_artifact_dir => "/tmp/artifacts",
        })
        .map_err(|e| e.to_string())
    }

    /// Render Synthesis Writer system prompt matching NVIDIA factory.py
    pub fn render_writer_system(
        &self,
        current_datetime: Option<&str>,
        user_info: Option<&Value>,
        parent_report_context_available: bool,
        chart_skill_enabled: bool,
        execution_enabled: bool,
    ) -> Result<String, String> {
        let tmpl = self.env.get_template("writer").map_err(|e| e.to_string())?;
        let now_str = current_datetime
            .map(|s| s.to_string())
            .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        let uinfo = user_info.cloned().unwrap_or_else(|| serde_json::json!({}));
        tmpl.render(context! {
            current_datetime => now_str,
            user_info => uinfo,
            parent_report_context_available => parent_report_context_available,
            chart_skill_enabled => chart_skill_enabled,
            execution_enabled => execution_enabled,
            sandbox_workdir => "/tmp/workspace",
            sandbox_artifact_dir => "/tmp/artifacts",
        })
        .map_err(|e| e.to_string())
    }

    /// Backward-compatible intent prompt renderer
    pub fn render_intent(&self, user_query: &str) -> Result<String, String> {
        let system_prompt = self.render_intent_system(None, None)?;
        Ok(format!("{}\n\nUser Query: {}", system_prompt, user_query))
    }

    /// Backward-compatible planner prompt renderer (now backed by full NVIDIA planner.j2)
    pub fn render_planner(&self, user_query: &str) -> Result<String, String> {
        let default_tools = vec![
            serde_json::json!({
                "name": "tavily_search",
                "description": "Comprehensive web search engine returning relevant snippets, titles, and verified URLs."
            }),
            serde_json::json!({
                "name": "think",
                "description": "Record reasoning steps, plan decomposition, evaluate evidence gaps, and verify constraints."
            }),
        ];
        let system_prompt = self.render_planner_system(
            &default_tools,
            false,
            6,
            None,
            None,
            None,
        )?;
        Ok(format!("{}\n\n## User Research Task\n{}", system_prompt, user_query))
    }

    /// Backward-compatible worker prompt renderer (backed by full NVIDIA researcher.j2)
    pub fn render_worker(&self, query: &str, target_angle: &str, search_content: &str) -> Result<String, String> {
        let default_tools = vec![
            serde_json::json!({
                "name": "tavily_search",
                "description": "Run targeted search queries for specific factual evidence and data points."
            }),
            serde_json::json!({
                "name": "think",
                "description": "Analyze results, identify information gaps, and evaluate confidence."
            }),
        ];
        let system_prompt = self.render_researcher_system(
            &default_tools,
            None,
            None,
            None,
            false,
        )?;
        Ok(format!(
            "{}\n\n## Assigned Research Query\n**Query:** {}\n**Target Angle:** {}\n\n## Search Findings\n{}",
            system_prompt, query, target_angle, search_content
        ))
    }

    /// Backward-compatible writer prompt renderer (backed by full NVIDIA writer.j2)
    pub fn render_writer(
        &self,
        user_query: &str,
        plan_title: &str,
        research_notes: &str,
        verified_sources: &Value,
    ) -> Result<String, String> {
        let system_prompt = self.render_writer_system(None, None, false, false, false)?;
        Ok(format!(
            "{}\n\n## User Request\n**Query:** {}\n**Research Plan Title:** {}\n\n## Compiled Research Notes & Evidence\n{}\n\n## Verified Sources Whitelist\n```json\n{}\n```",
            system_prompt,
            user_query,
            plan_title,
            research_notes,
            serde_json::to_string_pretty(verified_sources).unwrap_or_default()
        ))
    }
}
