// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use deep_researcher_agent::agents::state::{
    AnswerComponent, AnswerStrategy, Constraint, ResearchPlan, ResearchQuery,
    ResearchFinding, ResearchNotes, ResearchSource, EvidenceJudgment,
};
use deep_researcher_agent::prompts::PromptEngine;

#[test]
fn test_prompt_parity_planner_template() {
    let engine = PromptEngine::new();
    let tools = vec![
        serde_json::json!({"name": "tavily_search", "description": "Web search"}),
        serde_json::json!({"name": "think", "description": "Reasoning"}),
    ];
    let rendered = engine
        .render_planner_system(&tools, false, 6, Some("2026-11-01 12:00:00"), None, None)
        .expect("Failed to render planner system prompt");

    // Verify key NVIDIA sections from planner.j2 are preserved
    assert!(rendered.contains("You are the Research Planner Subagent."));
    assert!(rendered.contains("## Task Analysis"));
    assert!(rendered.contains("## Workflow"));
    assert!(rendered.contains("## Dynamic Discovery Budget"));
    assert!(rendered.contains("## Answer Strategy"));
    assert!(rendered.contains("## Constraints"));
    assert!(rendered.contains("## Queries"));
    assert!(rendered.contains("tavily_search"));
    assert!(rendered.contains("2026-11-01 12:00:00"));
}

#[test]
fn test_prompt_parity_researcher_template() {
    let engine = PromptEngine::new();
    let tools = vec![
        serde_json::json!({"name": "tavily_search", "description": "Web search"}),
        serde_json::json!({"name": "think", "description": "Reasoning"}),
    ];
    let rendered = engine
        .render_researcher_system(&tools, Some("2026-11-01 12:00:00"), None, None, false)
        .expect("Failed to render researcher system prompt");

    // Verify key NVIDIA sections from researcher.j2 are preserved
    assert!(rendered.contains("Gather and synthesize comprehensive information on the provided query"));
    assert!(rendered.contains("## Research Protocol"));
    assert!(rendered.contains("## Guidelines"));
    assert!(rendered.contains("## Depth Requirements"));
    assert!(rendered.contains("## Output Requirements"));
    assert!(rendered.contains("## Tool Use"));
    assert!(rendered.contains("## Handling Failures"));
    assert!(rendered.contains("ResearchNotes"));
    assert!(rendered.contains("tavily_search"));
}

#[test]
fn test_prompt_parity_writer_template() {
    let engine = PromptEngine::new();
    let rendered = engine
        .render_writer_system(Some("2026-11-01 12:00:00"), None, false, false, false)
        .expect("Failed to render writer system prompt");

    // Verify key NVIDIA sections from writer.j2 are preserved
    assert!(rendered.contains("You are the Writer Subagent. Your job is final synthesis, not research."));
    assert!(rendered.contains("## Skill Preflight"));
    assert!(rendered.contains("## Required Inputs"));
    assert!(rendered.contains("## Synthesis Contract"));
    assert!(rendered.contains("## General Cross-Synthesis Guidance"));
    assert!(rendered.contains("## Citations"));
}

#[test]
fn test_research_plan_contracts_serialization_roundtrip() {
    let plan = ResearchPlan {
        title: "Test Plan".to_string(),
        strategy: "Comprehensive".to_string(),
        task_analysis: Default::default(),
        answer_strategy: AnswerStrategy {
            answer_type: "long_form_report".to_string(),
            title: "Test Plan".to_string(),
            required_components: vec![AnswerComponent {
                id: "comp1".to_string(),
                name: "Component 1".to_string(),
                description: "Description 1".to_string(),
            }],
        },
        constraints: vec![Constraint {
            category: "depth".to_string(),
            constraint: "Must include numbers".to_string(),
            rationale: "Rigor".to_string(),
        }],
        queries: vec![ResearchQuery {
            query: "Test query".to_string(),
            subqueries: vec!["angle 1".to_string()],
            preferred_tools: vec!["tavily_search".to_string()],
            fallback_tools: vec![],
            target_components: vec!["comp1".to_string()],
            rationale: "Essential".to_string(),
        }],
        sub_queries: vec![],
    };

    let serialized = serde_json::to_string(&plan).expect("Failed to serialize");
    let deserialized: ResearchPlan = serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert_eq!(deserialized.title, "Test Plan");
    assert_eq!(deserialized.get_queries().len(), 1);
    assert_eq!(deserialized.answer_strategy.required_components.len(), 1);
    assert_eq!(deserialized.constraints.len(), 1);
}

#[test]
fn test_research_notes_contracts_serialization_roundtrip() {
    let notes = ResearchNotes {
        query_topic: "Streaming LLMs".to_string(),
        target_components: vec!["comp1".to_string()],
        summary: "Feasibility confirmed with SubSpec".to_string(),
        findings: vec![ResearchFinding {
            claim: "10x speedup with SubSpec".to_string(),
            evidence: "SubSpec paper NeurIPS 2025 demonstrates 10-12.5x speedup".to_string(),
            source_ids: vec![1],
            confidence: "high".to_string(),
            caveats: vec!["Requires 4-bit draft substitute".to_string()],
        }],
        gaps: vec![],
        sources: vec![ResearchSource {
            id: 1,
            title: "SubSpec Paper".to_string(),
            source_type: "url".to_string(),
            locator: "https://arxiv.org/abs/2509.18344".to_string(),
        }],
        narrative_notes: "Full architectural analysis of weight streaming...".to_string(),
        language: "English".to_string(),
        evidence_judgment: Some(EvidenceJudgment {
            relevance_score: 95,
            confidence: "high".to_string(),
            rationale: "Direct empirical benchmarks on 70B models".to_string(),
        }),
    };

    let serialized = serde_json::to_string(&notes).expect("Failed to serialize notes");
    let deserialized: ResearchNotes = serde_json::from_str(&serialized).expect("Failed to deserialize notes");

    assert_eq!(deserialized.query_topic, "Streaming LLMs");
    assert_eq!(deserialized.findings.len(), 1);
    assert_eq!(deserialized.sources.len(), 1);
    assert_eq!(deserialized.evidence_judgment.as_ref().unwrap().relevance_score, 95);
}
