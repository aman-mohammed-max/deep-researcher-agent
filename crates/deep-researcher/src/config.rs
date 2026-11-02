use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub runtime_database_url: String,
    pub postgres_url: String,
    pub persistent_database_url: String,

    // LLM Settings
    pub llm_base_url: String,
    pub llm_model: String,
    pub llm_api_key: String,
    #[allow(dead_code)]
    pub llm_temperature: f32,
    #[allow(dead_code)]
    pub llm_max_tokens: u32,

    // Search Tools
    pub tavily_api_key: Option<String>,
    #[allow(dead_code)]
    pub serper_api_key: Option<String>,

    // Resource Guard (Render 512 MB memory threshold)
    pub max_ram_percent: f32,
    pub max_concurrent_jobs: usize,
    pub max_research_concurrency: usize,
}

impl AppConfig {
    pub fn from_env() -> Self {
        // Load .env or deploy/.env if present
        let _ = dotenvy::from_filename("deploy/.env")
            .or_else(|_| dotenvy::from_filename(".env"))
            .or_else(|_| dotenvy::dotenv());

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8000);

        let runtime_database_url = env::var("RUNTIME_DATABASE_URL")
            .or_else(|_| env::var("EPHEMERAL_DATABASE_URL"))
            .unwrap_or_else(|_| "sqlite::memory:?cache=shared".to_string());

        let postgres_url = env::var("POSTGRES_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .or_else(|_| env::var("PERSISTENT_DATABASE_URL"))
            .unwrap_or_else(|_| "sqlite:deep_researcher_persistent.db?mode=rwc".to_string());

        let persistent_database_url = postgres_url.clone();
        let database_url = postgres_url.clone();

        // LLM configurations (checks NVIDIA_API_KEY, then OPENAI_API_KEY, then DEEPSEEK_API_KEY)
        let (default_base_url, default_model, api_key) = if let Ok(k) = env::var("NVIDIA_API_KEY") {
            (
                "https://integrate.api.nvidia.com/v1".to_string(),
                env::var("DEEP_RESEARCHER_MODEL").unwrap_or_else(|_| "nvidia/nemotron-3.5-lightning-30b-a3b".to_string()),
                k,
            )
        } else if let Ok(k) = env::var("DEEPSEEK_API_KEY") {
            (
                "https://api.deepseek.com/v1".to_string(),
                env::var("DEEP_RESEARCHER_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string()),
                k,
            )
        } else if let Ok(k) = env::var("OPENAI_API_KEY") {
            (
                "https://api.openai.com/v1".to_string(),
                env::var("DEEP_RESEARCHER_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
                k,
            )
        } else {
            (
                env::var("LLM_BASE_URL").unwrap_or_else(|_| "https://integrate.api.nvidia.com/v1".to_string()),
                env::var("DEEP_RESEARCHER_MODEL").unwrap_or_else(|_| "nvidia/nemotron-3.5-lightning-30b-a3b".to_string()),
                env::var("LLM_API_KEY").unwrap_or_default(),
            )
        };

        let llm_base_url = env::var("LLM_BASE_URL").unwrap_or(default_base_url);
        let llm_model = env::var("LLM_MODEL").unwrap_or(default_model);
        let llm_api_key = env::var("LLM_API_KEY").unwrap_or(api_key);

        let tavily_api_key = env::var("TAVILY_API_KEY").ok().filter(|s| !s.is_empty());
        let serper_api_key = env::var("SERPER_API_KEY").ok().filter(|s| !s.is_empty());

        let max_ram_percent = env::var("MAX_RAM_PERCENT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(85.0);

        let max_concurrent_jobs = env::var("MAX_CONCURRENT_JOBS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4);

        let max_research_concurrency = env::var("MAX_RESEARCH_CONCURRENCY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        Self {
            host,
            port,
            database_url,
            runtime_database_url,
            postgres_url,
            persistent_database_url,
            llm_base_url,
            llm_model,
            llm_api_key,
            llm_temperature: 0.2,
            llm_max_tokens: 8192,
            tavily_api_key,
            serper_api_key,
            max_ram_percent,
            max_concurrent_jobs,
            max_research_concurrency,
        }
    }
}
