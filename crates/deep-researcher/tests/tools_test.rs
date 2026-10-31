use deep_researcher_agent::tools::SearchDispatcher;

#[tokio::test]
async fn test_search_dispatcher_offline_safe_fallback() {
    // When no Tavily key is supplied, dispatcher falls back gracefully to DuckDuckGo / Reddit
    let dispatcher = SearchDispatcher::new(None);
    let results = dispatcher.search("Rust async programming", "technical", 3).await;

    // Search should always return at least a graceful fallback or empty vec, never panic
    assert!(results.len() <= 3);
}

#[tokio::test]
async fn test_search_dispatcher_community_angle() {
    let dispatcher = SearchDispatcher::new(None);
    // When target angle is community_sentiment, dispatcher routes to reddit first
    let results = dispatcher.search("Rental rooms in Kochi", "community_sentiment", 5).await;
    assert!(results.len() <= 5);
}
