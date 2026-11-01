// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use deep_researcher_agent::prompts::PromptEngine;

#[test]
fn test_render_intent_prompt() {
    let engine = PromptEngine::new();
    let rendered = engine.render_intent("Find affordable room for rent in Kochi").expect("Failed to render intent prompt");

    assert!(rendered.contains("Find affordable room for rent in Kochi"));
    assert!(rendered.contains("intent"));
    assert!(rendered.contains("research"));
}

#[test]
fn test_render_planner_prompt() {
    let engine = PromptEngine::new();
    let rendered = engine.render_planner("Market analysis of NVIDIA GPU sales 2026").expect("Failed to render planner prompt");

    assert!(rendered.contains("Market analysis of NVIDIA GPU sales 2026"));
    assert!(rendered.contains("Research Planner Subagent"));
    assert!(rendered.contains("ResearchPlan"));
    assert!(rendered.contains("Task Decomposition"));
    assert!(rendered.contains("answer_strategy"));
}

#[test]
fn test_render_worker_prompt() {
    let engine = PromptEngine::new();
    let search_content = "Rental listings in Kakkanad: 1BHK 12k INR per month with deposit.";
    let rendered = engine.render_worker(
        "Kochi Kakkanad rental flats",
        "pricing_local",
        search_content,
    ).expect("Failed to render worker prompt");

    assert!(rendered.contains("Kochi Kakkanad rental flats"));
    assert!(rendered.contains("pricing_local"));
    assert!(rendered.contains("Rental listings in Kakkanad"));
    assert!(rendered.contains("Research Protocol"));
    assert!(rendered.contains("ResearchNotes"));
}

#[test]
fn test_render_writer_prompt() {
    let engine = PromptEngine::new();
    let notes = "Sub-query 1: Kakkanad IT corridor has rooms starting at 10k.\nSub-query 2: Edappally metro access rooms average 15k.";
    let sources = serde_json::json!([
        {
            "title": "Kochi Reddit Rental Discussion",
            "url": "https://reddit.com/r/kochi/comments/rent",
            "source_type": "reddit"
        }
    ]);
    
    let rendered = engine.render_writer(
        "Find room in Kochi",
        "Comprehensive Kochi Room Rental Analysis",
        notes,
        &sources,
    ).expect("Failed to render writer prompt");

    assert!(rendered.contains("Find room in Kochi"));
    assert!(rendered.contains("Comprehensive Kochi Room Rental Analysis"));
    assert!(rendered.contains("Kakkanad IT corridor"));
    assert!(rendered.contains("reddit.com"));
    assert!(rendered.contains("Writer Subagent"));
    assert!(rendered.contains("Synthesis Contract"));
    assert!(rendered.contains("Citations"));
}

#[test]
fn test_nvidia_unabridged_system_templates_registered() {
    let engine = PromptEngine::new();
    
    let tools = vec![
        serde_json::json!({"name": "tavily_search", "description": "Web search"}),
        serde_json::json!({"name": "think", "description": "Reasoning"}),
    ];

    let planner_sys = engine.render_planner_system(&tools, false, 6, None, None, None).unwrap();
    assert!(planner_sys.contains("You are the Research Planner Subagent"));
    assert!(planner_sys.contains("Dynamic Discovery Budget"));
    assert!(planner_sys.contains("tavily_search"));

    let worker_sys = engine.render_researcher_system(&tools, None, None, None, false).unwrap();
    assert!(worker_sys.contains("Gather and synthesize comprehensive information"));
    assert!(worker_sys.contains("Research Protocol"));
    assert!(worker_sys.contains("Depth Requirements"));

    let writer_sys = engine.render_writer_system(None, None, false, false, false).unwrap();
    assert!(writer_sys.contains("You are the Writer Subagent. Your job is final synthesis, not research."));
    assert!(writer_sys.contains("General Cross-Synthesis Guidance"));
}
