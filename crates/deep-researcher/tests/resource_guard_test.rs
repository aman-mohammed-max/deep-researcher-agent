use deep_researcher_agent::resource_guard::{AdmissionStatus, ResourceGuard, SystemResourceMetrics};

#[test]
fn test_system_resource_metrics_read() {
    let metrics = SystemResourceMetrics::read_current(0, 0);
    println!("Read system metrics: {:?}", metrics);
    // On Linux systems, total RAM is > 0
    assert!(metrics.total_ram_mb > 0);
    assert!(metrics.ram_usage_percent >= 0.0 && metrics.ram_usage_percent <= 100.0);
}

#[tokio::test]
async fn test_resource_guard_admission_and_backpressure() {
    // Set max_ram_percent to 99% and max concurrent jobs to 2
    let guard = ResourceGuard::new(99.0, 2);

    // Initial check: should be admitted
    let status_1 = guard.check_admission("job_1").await;
    assert!(matches!(status_1, AdmissionStatus::Admitted(_)));

    let permit_1 = match status_1 {
        AdmissionStatus::Admitted(p) => p,
        _ => panic!("Expected permit 1 to be admitted"),
    };
    assert_eq!(guard.get_metrics().await.active_jobs, 1);

    // Second check: should also be admitted
    let status_2 = guard.check_admission("job_2").await;
    assert!(matches!(status_2, AdmissionStatus::Admitted(_)));

    let permit_2 = match status_2 {
        AdmissionStatus::Admitted(p) => p,
        _ => panic!("Expected permit 2 to be admitted"),
    };
    assert_eq!(guard.get_metrics().await.active_jobs, 2);

    // Third check: concurrency limit reached (2/2), must be Queued
    let status_3 = guard.check_admission("job_3").await;
    match status_3 {
        AdmissionStatus::Queued { position, estimated_wait_secs, .. } => {
            assert_eq!(position, 1);
            assert!(estimated_wait_secs > 0);
        }
        AdmissionStatus::Admitted(_) => panic!("Expected job 3 to be held in queue"),
    }

    // Drop permit 1 (RAII cleanup)
    drop(permit_1);
    assert_eq!(guard.get_metrics().await.active_jobs, 1);

    // Now job 4 should be admitted
    let status_4 = guard.check_admission("job_4").await;
    assert!(matches!(status_4, AdmissionStatus::Admitted(_)));
    assert_eq!(guard.get_metrics().await.active_jobs, 2);

    drop(permit_2);
    drop(status_4);
    assert_eq!(guard.get_metrics().await.active_jobs, 0);
}

#[tokio::test]
async fn test_resource_guard_extreme_low_ram_threshold() {
    // Set max_ram_percent to 0.001% so system RAM will exceed it
    let guard = ResourceGuard::new(0.001, 10);
    let status = guard.check_admission("job_low_thresh").await;

    match status {
        AdmissionStatus::Queued { position, .. } => {
            assert_eq!(position, 1);
        }
        AdmissionStatus::Admitted(_) => panic!("Expected to be queued due to low threshold"),
    }
}
