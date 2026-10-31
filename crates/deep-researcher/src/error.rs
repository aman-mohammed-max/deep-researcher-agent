#[allow(dead_code)]
#[derive(Debug)]
pub enum AppError {
    Config(String),
    Llm(String),
    Network(String),
    Storage(String),
    ResourceExhausted(String),
    Tool(String),
    Skill(String),
    Json(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AppError::Llm(msg) => write!(f, "LLM error: {}", msg),
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Storage(msg) => write!(f, "Storage error: {}", msg),
            AppError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
            AppError::Tool(msg) => write!(f, "Tool execution error: {}", msg),
            AppError::Skill(msg) => write!(f, "Skill error: {}", msg),
            AppError::Json(msg) => write!(f, "JSON serialization error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Json(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Storage(err.to_string())
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::ResourceExhausted(msg) => (axum::http::StatusCode::SERVICE_UNAVAILABLE, msg),
            AppError::Config(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            AppError::Llm(msg) => (axum::http::StatusCode::BAD_GATEWAY, msg),
            AppError::Network(msg) => (axum::http::StatusCode::BAD_GATEWAY, msg),
            AppError::Storage(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Tool(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Skill(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Json(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
        };

        let body = axum::Json(serde_json::json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
