use minijinja::{context, Environment};
use serde_json::Value;

pub const INTENT_PROMPT: &str = r#"
You are the orchestration layer of an AI Research Assistant. Your goal is to analyze the user's query, determine its intent, and provide either a direct response (Meta) or route to research (Shallow/Deep).

### STEP 1: INTENT CLASSIFICATION
Classify the query as "meta" or "research".
- **meta**: System identity, greetings, time/date, tool questions, emotional check-ins, jokes, casual chat, small talk.
- **research**: Facts, data, analysis, "how-to", scientific/technical topics, local inquiries (rentals, travel, places), or anything requiring external data/sources.
- **Rule**: If a query is mixed or unsure, choose "research".

### STEP 2: DEPTH DETERMINATION (For research)
- **shallow**: Simple factual lookup, single question, 1-2 sources needed.
- **deep**: In-depth investigations, reports, comparisons, multi-faceted requests, comprehensive guides, or finding items/places across multiple platforms.

### OUTPUT FORMAT
You must respond ONLY with a JSON object in this exact schema:
{
  "intent": "meta" | "research",
  "meta_response": "The friendly text response if intent is meta, otherwise null",
  "research_depth": "shallow" | "deep" | null,
  "reasoning": "One sentence explanation"
}

Current Date and Time: {{ current_datetime }}
User Query: {{ user_query }}
"#;

pub const PLANNER_PROMPT: &str = r#"
You are the Research Planner Agent. Your job is to analyze the user request and generate an evidence-grounded research plan.

Break the user request down into 2 to 5 targeted sub-queries that can be researched concurrently.
For each sub-query, specify what kind of information is sought (e.g. general web overview, community discussions/sentiment on Reddit or X, pricing/data, or official specs).

User Query: {{ user_query }}

Respond ONLY with a JSON object in this schema:
{
  "title": "Title of the research task",
  "strategy": "Overview of research approach",
  "sub_queries": [
    {
      "query": "Specific targeted search query",
      "target_angle": "community_sentiment" | "web_facts" | "pricing_local" | "technical_data",
      "purpose": "Why this query is essential"
    }
  ]
}
"#;

pub const WORKER_PROMPT: &str = r#"
You are a Research Worker Agent. Your job is to extract concrete facts, figures, direct evidence, user sentiments, and citations from search results for a specific research query.

Sub-Query Target: {{ query }}
Target Angle: {{ target_angle }}

Raw Search Results & Content:
{{ search_content }}

Extract the key findings, including verified URLs, quotes, numbers, pros/cons, and community consensus.
Respond with a structured summary highlighting:
1. Core Findings & Direct Evidence
2. Specific URLs & Source Citations
3. Limitations or Information Gaps encountered
"#;

pub const WRITER_PROMPT: &str = r#"
You are the Lead Research Report Writer. Your job is to synthesize all research findings into a comprehensive, professional, well-structured Markdown report.

User Original Query: {{ user_query }}
Research Plan Title: {{ plan_title }}

Collected Research Notes & Evidence:
{{ research_notes }}

Verified Sources Captured:
{{ verified_sources }}

Instructions for Synthesis:
1. Write a clear, executive-grade Markdown document with appropriate headings (#, ##, ###).
2. Incorporate specific numbers, dates, prices, direct quotes, and community consensus found in the research notes.
3. Every major factual claim or recommendation MUST cite its corresponding source using Markdown links [Source Name](URL).
4. If information for certain parts was limited or unavailable, honestly describe the gap.
5. End with a "Summary & Recommendations" section.
"#;

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
        env.add_template("intent", INTENT_PROMPT).unwrap();
        env.add_template("planner", PLANNER_PROMPT).unwrap();
        env.add_template("worker", WORKER_PROMPT).unwrap();
        env.add_template("writer", WRITER_PROMPT).unwrap();
        Self { env }
    }

    pub fn render_intent(&self, user_query: &str) -> Result<String, String> {
        let tmpl = self.env.get_template("intent").map_err(|e| e.to_string())?;
        let current_datetime = chrono::Utc::now().to_rfc3339();
        tmpl.render(context! { user_query, current_datetime })
            .map_err(|e| e.to_string())
    }

    pub fn render_planner(&self, user_query: &str) -> Result<String, String> {
        let tmpl = self.env.get_template("planner").map_err(|e| e.to_string())?;
        tmpl.render(context! { user_query }).map_err(|e| e.to_string())
    }

    pub fn render_worker(&self, query: &str, target_angle: &str, search_content: &str) -> Result<String, String> {
        let tmpl = self.env.get_template("worker").map_err(|e| e.to_string())?;
        tmpl.render(context! { query, target_angle, search_content })
            .map_err(|e| e.to_string())
    }

    pub fn render_writer(
        &self,
        user_query: &str,
        plan_title: &str,
        research_notes: &str,
        verified_sources: &Value,
    ) -> Result<String, String> {
        let tmpl = self.env.get_template("writer").map_err(|e| e.to_string())?;
        tmpl.render(context! { user_query, plan_title, research_notes, verified_sources })
            .map_err(|e| e.to_string())
    }
}
