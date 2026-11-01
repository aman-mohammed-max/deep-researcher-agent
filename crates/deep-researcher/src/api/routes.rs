use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse::Event, IntoResponse, Sse},
    Json,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use crate::agents::ResearchCoordinator;
use crate::error::AppError;
use crate::resource_guard::{AdmissionStatus, ResourceGuard};
use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub coordinator: Arc<ResearchCoordinator>,
    pub resource_guard: ResourceGuard,
    pub storage: Storage,
}

#[derive(Debug, Deserialize)]
pub struct SubmitJobRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitJobResponse {
    pub job_id: String,
    pub status: String,
    pub message: Option<String>,
    pub queue_position: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
}

/// GET /health
pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let metrics = state.resource_guard.get_metrics().await;
    Json(serde_json::json!({
        "status": "healthy",
        "system_ram_pct": metrics.ram_usage_percent,
        "process_rss_mb": metrics.process_rss_mb,
        "free_ram_mb": metrics.free_ram_mb,
        "active_jobs": metrics.active_jobs,
        "queue_length": metrics.queue_length,
        "db": "ok"
    }))
}

/// GET /v1/data_sources
pub async fn data_sources_handler() -> impl IntoResponse {
    Json(serde_json::json!([
        {
            "id": "web_search",
            "name": "Web Search (Tavily/DDG)",
            "description": "Real-time web search across verified news and technical sources.",
            "default_enabled": true,
            "requires_auth": false,
            "per_user_auth": null
        },
        {
            "id": "reddit_search",
            "name": "Reddit & Community Discussions",
            "description": "Zero-auth deep research on subreddits, real user reviews, and forum threads.",
            "default_enabled": true,
            "requires_auth": false,
            "per_user_auth": null
        },
        {
            "id": "marketplace_skills",
            "name": "Autonomous Skills & MCP Marketplace",
            "description": "Dynamically discovers and activates public, zero-auth scrapers and domain tools.",
            "default_enabled": true,
            "requires_auth": false,
            "per_user_auth": null
        }
    ]))
}

/// POST /v1/jobs/async
pub async fn submit_job_handler(
    State(state): State<AppState>,
    Json(payload): Json<SubmitJobRequest>,
) -> Result<Json<SubmitJobResponse>, AppError> {
    let job_id = uuid::Uuid::new_v4().to_string();
    let query = payload.query.trim().to_string();

    if query.is_empty() {
        return Err(AppError::Config("Query cannot be empty".to_string()));
    }

    state.storage.create_job(&job_id, &query).await?;

    let admission = state.resource_guard.check_admission(&job_id).await;

    match admission {
        AdmissionStatus::Admitted(permit) => {
            let coordinator = Arc::clone(&state.coordinator);
            let j_id = job_id.clone();
            let q = query.clone();

            tokio::spawn(async move {
                let _permit = permit; // RAII release on task finish
                if let Err(err) = coordinator.run_workflow(&j_id, &q).await {
                    tracing::error!("Job '{}' workflow failed: {}", j_id, err);
                    let _ = coordinator.storage.fail_job(&j_id, &err.to_string()).await;
                    let _ = coordinator.storage.add_event(&j_id, "error", &format!("Research failed: {}", err)).await;
                }
            });

            Ok(Json(SubmitJobResponse {
                job_id,
                status: "RUNNING".to_string(),
                message: None,
                queue_position: None,
            }))
        }
        AdmissionStatus::Queued {
            position,
            estimated_wait_secs,
            ticket,
        } => {
            let coordinator = Arc::clone(&state.coordinator);
            let guard = state.resource_guard.clone();
            let storage = state.storage.clone();
            let j_id = job_id.clone();
            let q = query.clone();

            let queue_msg = format!(
                "Server memory near watermark. Your query is queued at position #{} and will start shortly (estimated wait: ~{}s)...",
                position, estimated_wait_secs
            );

            let _ = storage.add_event(&j_id, "queued", &queue_msg).await;

            // Background task awaits admission
            tokio::spawn(async move {
                match guard.wait_for_admission(ticket, Duration::from_secs(300)).await {
                    Ok(permit) => {
                        let _permit = permit;
                        let _ = storage.add_event(&j_id, "dequeued", "Memory freed. Starting research now.").await;
                        if let Err(err) = coordinator.run_workflow(&j_id, &q).await {
                            tracing::error!("Job '{}' workflow failed: {}", j_id, err);
                            let _ = coordinator.storage.fail_job(&j_id, &err.to_string()).await;
                            let _ = coordinator.storage.add_event(&j_id, "error", &format!("Research failed: {}", err)).await;
                        }
                    }
                    Err(err) => {
                        let _ = storage.fail_job(&j_id, &err).await;
                    }
                }
            });

            Ok(Json(SubmitJobResponse {
                job_id,
                status: "QUEUED_ON_HOLD".to_string(),
                message: Some(queue_msg),
                queue_position: Some(position),
            }))
        }
    }
}

/// GET /v1/jobs/async/:id
pub async fn get_job_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let job = state.storage.get_job(&id).await?;
    match job {
        Some(j) => Ok(Json(serde_json::to_value(j)?)),
        None => Err(AppError::Config(format!("Job '{}' not found", id))),
    }
}

/// GET /v1/jobs/async/:id/events (SSE Stream)
pub async fn job_events_stream_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let storage = state.storage.clone();

    let stream = async_stream::stream! {
        let mut last_event_id = 0i64;

        loop {
            if let Ok(events) = storage.get_events(&id).await {
                for ev in events {
                    if ev.id > last_event_id {
                        last_event_id = ev.id;
                        yield Ok(Event::default().json_data(ev).unwrap_or_default());
                    }
                }
            }

            if let Ok(Some(job)) = storage.get_job(&id).await {
                if job.status == "COMPLETED" || job.status == "FAILED" {
                    yield Ok(Event::default().event("end").data(job.status));
                    break;
                }
            }

            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}

/// GET /api/dashboard/stats
pub async fn dashboard_stats_handler(State(state): State<AppState>) -> impl IntoResponse {
    let metrics = state.resource_guard.get_metrics().await;
    let jobs = state.storage.list_jobs(50).await.unwrap_or_default();

    let mut total_tokens = 0i64;
    let mut completed = 0;
    let mut failed = 0;

    for j in &jobs {
        total_tokens += j.total_tokens;
        if j.status == "COMPLETED" {
            completed += 1;
        } else if j.status == "FAILED" {
            failed += 1;
        }
    }

    Json(serde_json::json!({
        "metrics": metrics,
        "tokenomics": {
            "total_tokens_consumed": total_tokens,
            "estimated_cost_usd": (total_tokens as f64 / 1_000_000.0) * 0.70, // ~$0.70 per 1M tokens
        },
        "jobs_summary": {
            "total": jobs.len(),
            "completed": completed,
            "failed": failed,
            "active": metrics.active_jobs,
            "queued": metrics.queue_length
        }
    }))
}

/// GET /api/dashboard/jobs
pub async fn dashboard_jobs_handler(State(state): State<AppState>) -> impl IntoResponse {
    let jobs = state.storage.list_jobs(100).await.unwrap_or_default();
    Json(jobs)
}

/// POST /api/keys
pub async fn create_api_key_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateKeyRequest>,
) -> Result<impl IntoResponse, AppError> {
    let key = state.storage.create_api_key(&payload.name).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "api_key": key, "name": payload.name }))))
}

/// GET /api/keys
pub async fn list_api_keys_handler(State(state): State<AppState>) -> impl IntoResponse {
    let keys = state.storage.list_api_keys().await.unwrap_or_default();
    Json(keys)
}

/// DELETE /api/keys/:id
pub async fn revoke_api_key_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.storage.revoke_api_key(&id).await?;
    Ok(Json(serde_json::json!({ "revoked": true, "id": id })))
}
