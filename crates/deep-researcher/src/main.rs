use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use deep_researcher_agent::agents::ResearchCoordinator;
use deep_researcher_agent::api::{create_router, AppState};
use deep_researcher_agent::config::AppConfig;
use deep_researcher_agent::llm::LlmClient;
use deep_researcher_agent::prompts::PromptEngine;
use deep_researcher_agent::resource_guard::ResourceGuard;
use deep_researcher_agent::skills::MarketplaceRegistry;
use deep_researcher_agent::storage::Storage;
use deep_researcher_agent::tools::SearchDispatcher;
use deep_researcher_agent::web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "deep_researcher=info,tower_http=info".into()),
        )
        .init();

    info!("Starting Deep Researcher Rust Engine (Render 512MB Optimized)...");

    // 2. Load Configuration
    let config = AppConfig::from_env();
    info!(
        "Configuration: host={}, port={}, model={}, max_ram_threshold={}%",
        config.host, config.port, config.llm_model, config.max_ram_percent
    );

    // 3. Initialize Tiered Storage Engine (RAM/In-Memory SQLite for Jobs & PostgreSQL for Keys)
    let storage = Storage::new(&config.runtime_database_url, &config.postgres_url).await?;
    info!(
        "Storage: Runtime (RAM/Jobs) = {} [in-memory: {}], Persistent (Keys) = {} [{}]",
        config.runtime_database_url,
        storage.is_in_memory_runtime(),
        config.postgres_url,
        storage.persistent_type()
    );

    // 4. Initialize Resource Guard (OOM Prevention & Backpressure)
    let resource_guard = ResourceGuard::new(config.max_ram_percent, config.max_concurrent_jobs);

    // 5. Initialize AI & Tooling Engine
    let llm = Arc::new(LlmClient::new(
        config.llm_base_url,
        config.llm_api_key,
        config.llm_model,
    ));

    let prompts = Arc::new(PromptEngine::new());
    let searcher = Arc::new(SearchDispatcher::new(config.tavily_api_key));
    let marketplace = Arc::new(MarketplaceRegistry::default());

    let coordinator = Arc::new(ResearchCoordinator::new(
        llm,
        prompts,
        searcher,
        marketplace,
        storage.clone(),
        config.max_research_concurrency,
    ));

    let app_state = AppState {
        coordinator,
        resource_guard,
        storage,
    };

    // 6. Build Axum Router + Embedded Frontend Fallback
    let router = create_router(app_state).fallback(web::static_handler);

    // 7. Bind & Serve
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("Deep Researcher server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
