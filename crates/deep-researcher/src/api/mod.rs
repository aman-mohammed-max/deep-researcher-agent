pub mod routes;
pub mod websocket;

use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub use routes::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Core API Endpoints
        .route("/health", get(routes::health_handler))
        .route("/v1/data_sources", get(routes::data_sources_handler))
        .route("/v1/jobs/async", post(routes::submit_job_handler))
        .route("/v1/jobs/async/{id}", get(routes::get_job_handler))
        .route("/v1/jobs/async/{id}/events", get(routes::job_events_stream_handler))
        .route("/websocket", get(websocket::websocket_handler))
        // Observability Dashboard & Playground API
        .route("/api/dashboard/stats", get(routes::dashboard_stats_handler))
        .route("/v1/system/metrics", get(routes::dashboard_stats_handler))
        .route("/api/dashboard/jobs", get(routes::dashboard_jobs_handler))
        .route("/api/keys", post(routes::create_api_key_handler).get(routes::list_api_keys_handler))
        .route("/v1/keys", post(routes::create_api_key_handler).get(routes::list_api_keys_handler))
        .route("/api/keys/{id}", delete(routes::revoke_api_key_handler))
        .route("/v1/keys/{id}", delete(routes::revoke_api_key_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
