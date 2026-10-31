use deep_researcher_agent::prompts::PromptEngine;

#[test]
fn test_render_intent_prompt() {
    let engine = PromptEngine::new();
    let rendered = engine.render_intent("Find affordable room for rent in Kochi").expect("Failed to render intent prompt");

    assert!(rendered.contains("Find affordable room for rent in Kochi"));
    assert!(rendered.contains("research"));
    assert!(rendered.contains("JSON object"));
}

#[test]
fn test_render_planner_prompt() {
    let engine = PromptEngine::new();
    let rendered = engine.render_planner("Market analysis of NVIDIA GPU sales 2026").expect("Failed to render planner prompt");

    assert!(rendered.contains("Market analysis of NVIDIA GPU sales 2026"));
    assert!(rendered.contains("sub_queries"));
    assert!(rendered.contains("target_angle"));
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
    assert!(rendered.contains("Summary & Recommendations"));
}
