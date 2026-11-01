// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use deep_researcher_agent::storage::Storage;

#[tokio::test]
async fn test_storage_job_lifecycle_and_events() {
    let storage = Storage::from_url("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite storage");

    let job_id = "test-job-123";
    let user_query = "Research renewable energy breakthroughs in 2026";

    // 1. Create job
    storage
        .create_job(job_id, user_query)
        .await
        .expect("Failed to create job");

    let job = storage
        .get_job(job_id)
        .await
        .expect("Failed to query job")
        .expect("Job should exist");

    assert_eq!(job.id, job_id);
    assert_eq!(job.query, user_query);
    assert_eq!(job.status, "QUEUED");

    // 2. Add events
    storage
        .add_event(job_id, "step", "Decomposing query...")
        .await
        .expect("Failed to add event 1");
    storage
        .add_event(job_id, "plan_created", "{\"title\": \"Energy Research\"}")
        .await
        .expect("Failed to add event 2");

    let events = storage
        .get_events(job_id)
        .await
        .expect("Failed to fetch events");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event_type, "step");
    assert_eq!(events[1].event_type, "plan_created");

    // 3. Complete job with tokenomics
    let result_report = "# Comprehensive Energy Report\nSolar efficiency reached 35%.";
    storage
        .complete_job(job_id, result_report, 1500, 800, 2300)
        .await
        .expect("Failed to complete job");

    let completed_job = storage
        .get_job(job_id)
        .await
        .expect("Failed to query job")
        .expect("Job should exist");

    assert_eq!(completed_job.status, "COMPLETED");
    assert_eq!(completed_job.report.as_deref(), Some(result_report));
    assert_eq!(completed_job.prompt_tokens, 1500);
    assert_eq!(completed_job.completion_tokens, 800);
    assert_eq!(completed_job.total_tokens, 2300);

    // 4. List jobs
    let all_jobs = storage.list_jobs(10).await.expect("Failed to list jobs");
    assert_eq!(all_jobs.len(), 1);
    assert_eq!(all_jobs[0].id, job_id);
}

#[tokio::test]
async fn test_storage_api_keys() {
    let storage = Storage::from_url("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite storage");

    // Create API key
    let raw_key = storage
        .create_api_key("Development Key")
        .await
        .expect("Failed to create API key");

    assert!(raw_key.starts_with("dr_"));

    // Verify API key
    let is_valid = storage
        .verify_api_key(&raw_key)
        .await
        .expect("Failed to verify API key");
    assert!(is_valid);

    // Verify non-existent key
    let invalid = storage
        .verify_api_key("invalid_key_xyz")
        .await
        .expect("Verification query failed");
    assert!(!invalid);

    // List API keys
    let keys = storage.list_api_keys().await.expect("Failed to list keys");
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].name, "Development Key");
}

#[tokio::test]
async fn test_tiered_dual_storage_architecture() {
    // Runtime = RAM SQLite (ephemeral memory for jobs)
    // Persistent = SQLite (simulating durable SQL for API keys)
    let storage = Storage::new(
        "sqlite::memory:",
        "sqlite::memory:",
    )
    .await
    .expect("Failed to initialize tiered storage");

    assert!(storage.is_in_memory_runtime());
    assert_eq!(storage.persistent_type(), "SQLite");

    // 1. Create a job in runtime RAM
    storage
        .create_job("runtime-job-001", "Simulated quantum algorithm research")
        .await
        .expect("Failed to create job in runtime storage");

    let job = storage
        .get_job("runtime-job-001")
        .await
        .expect("Query failed")
        .expect("Job should exist in runtime RAM");
    assert_eq!(job.query, "Simulated quantum algorithm research");

    // 2. Create persistent API key
    let key = storage
        .create_api_key("Production Telegram Bridge")
        .await
        .expect("Failed to create key");
    assert!(key.starts_with("dr_"));

    let verified = storage.verify_api_key(&key).await.expect("Key check failed");
    assert!(verified);
}

#[tokio::test]
async fn test_postgres_url_driver_routing() {
    // Verifies that a postgres:// URL routes directly into the PostgreSQL driver pool
    let result = Storage::new(
        "sqlite::memory:",
        "postgres://testuser:testpass@127.0.0.1:5432/deep_researcher_test",
    )
    .await;

    // Because there is no PostgreSQL daemon running on 127.0.0.1:5432 in this container,
    // sqlx::postgres must return a connection error (confirming PostgreSQL driver engagement).
    assert!(result.is_err(), "Expected connection error to unhosted PostgreSQL database");
    let err_str = result.err().unwrap().to_string();
    assert!(
        err_str.contains("pool timed out")
            || err_str.contains("Connection refused")
            || err_str.contains("error connecting"),
        "Expected PostgreSQL connection or timeout error, got: {}",
        err_str
    );
}
