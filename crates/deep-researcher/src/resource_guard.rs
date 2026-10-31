use std::collections::VecDeque;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use tracing::{info, warn};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemResourceMetrics {
    pub total_ram_mb: u64,
    pub used_ram_mb: u64,
    pub free_ram_mb: u64,
    pub ram_usage_percent: f32,
    pub process_rss_mb: u64,
    pub process_ram_percent: f32,
    pub cpu_usage_percent: f32,
    pub active_jobs: usize,
    pub queue_length: usize,
}

impl SystemResourceMetrics {
    /// Read zero-overhead Linux kernel metrics from /proc/meminfo and /proc/self/statm
    pub fn read_current(active_jobs: usize, queue_length: usize) -> Self {
        #[cfg(target_os = "linux")]
        {
            let (total_ram_mb, used_ram_mb, free_ram_mb, ram_pct) = Self::read_linux_meminfo();
            let process_rss_mb = Self::read_linux_process_rss();
            let process_pct = if total_ram_mb > 0 {
                (process_rss_mb as f32 / total_ram_mb as f32) * 100.0
            } else {
                0.0
            };

            Self {
                total_ram_mb,
                used_ram_mb,
                free_ram_mb,
                ram_usage_percent: ram_pct,
                process_rss_mb,
                process_ram_percent: process_pct,
                cpu_usage_percent: 0.0, // Calculated on interval if needed
                active_jobs,
                queue_length,
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self {
                total_ram_mb: 512,
                used_ram_mb: 120,
                free_ram_mb: 392,
                ram_usage_percent: 23.4,
                process_rss_mb: 45,
                process_ram_percent: 8.7,
                cpu_usage_percent: 5.0,
                active_jobs,
                queue_length,
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn read_linux_meminfo() -> (u64, u64, u64, f32) {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            let mut total_kb = 0u64;
            let mut avail_kb = 0u64;

            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    total_kb = parse_kb_line(line);
                } else if line.starts_with("MemAvailable:") {
                    avail_kb = parse_kb_line(line);
                }
            }

            if total_kb > 0 {
                let total_mb = total_kb / 1024;
                let free_mb = avail_kb / 1024;
                let used_mb = total_mb.saturating_sub(free_mb);
                let pct = (used_mb as f32 / total_mb as f32) * 100.0;
                return (total_mb, used_mb, free_mb, pct);
            }
        }
        (512, 100, 412, 19.5)
    }

    #[cfg(target_os = "linux")]
    fn read_linux_process_rss() -> u64 {
        if let Ok(content) = fs::read_to_string("/proc/self/statm") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(pages) = parts[1].parse::<u64>() {
                    // Standard Linux page size is 4096 bytes
                    return (pages * 4096) / (1024 * 1024);
                }
            }
        }
        0
    }
}

#[cfg(target_os = "linux")]
fn parse_kb_line(line: &str) -> u64 {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse::<u64>().unwrap_or(0)
    } else {
        0
    }
}

pub enum AdmissionStatus {
    Admitted(JobPermit),
    Queued {
        position: usize,
        estimated_wait_secs: u64,
        ticket: Arc<Notify>,
    },
}

pub struct JobPermit {
    guard: Arc<ResourceGuardInner>,
}

impl Drop for JobPermit {
    fn drop(&mut self) {
        self.guard.release();
    }
}

struct ResourceGuardInner {
    max_ram_percent: f32,
    max_concurrent_jobs: usize,
    active_jobs: AtomicUsize,
    queue: Mutex<VecDeque<(String, Arc<Notify>)>>,
}

impl ResourceGuardInner {
    fn release(&self) {
        let prev = self.active_jobs.fetch_sub(1, Ordering::SeqCst);
        info!("Job completed. Active jobs decremented: {} -> {}", prev, prev.saturating_sub(1));

        // Attempt to wake the next queued job if under memory watermark
        let metrics = SystemResourceMetrics::read_current(
            self.active_jobs.load(Ordering::SeqCst),
            0,
        );

        if metrics.ram_usage_percent < self.max_ram_percent {
            let queue_lock = self.queue.try_lock();
            if let Ok(mut q) = queue_lock {
                if let Some((job_id, notify)) = q.pop_front() {
                    info!("Waking queued research job '{}' from backpressure hold", job_id);
                    notify.notify_one();
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ResourceGuard {
    inner: Arc<ResourceGuardInner>,
}

impl ResourceGuard {
    pub fn new(max_ram_percent: f32, max_concurrent_jobs: usize) -> Self {
        Self {
            inner: Arc::new(ResourceGuardInner {
                max_ram_percent,
                max_concurrent_jobs,
                active_jobs: AtomicUsize::new(0),
                queue: Mutex::new(VecDeque::new()),
            }),
        }
    }

    pub async fn get_metrics(&self) -> SystemResourceMetrics {
        let active = self.inner.active_jobs.load(Ordering::SeqCst);
        let q_len = self.inner.queue.lock().await.len();
        SystemResourceMetrics::read_current(active, q_len)
    }

    /// Check if job can run immediately or must wait in queue
    pub async fn check_admission(&self, job_id: &str) -> AdmissionStatus {
        let metrics = self.get_metrics().await;
        let active = self.inner.active_jobs.load(Ordering::SeqCst);

        let over_ram_watermark = metrics.ram_usage_percent >= self.inner.max_ram_percent;
        let over_job_concurrency = active >= self.inner.max_concurrent_jobs;

        if !over_ram_watermark && !over_job_concurrency {
            self.inner.active_jobs.fetch_add(1, Ordering::SeqCst);
            info!(
                "Job '{}' admitted immediately. Active jobs: {} (RAM: {:.1}%, Process RSS: {}MB)",
                job_id,
                active + 1,
                metrics.ram_usage_percent,
                metrics.process_rss_mb
            );
            AdmissionStatus::Admitted(JobPermit {
                guard: Arc::clone(&self.inner),
            })
        } else {
            warn!(
                "Backpressure triggered for job '{}'! Over watermark (RAM: {:.1}%, Active: {}/{}). Placing on hold.",
                job_id, metrics.ram_usage_percent, active, self.inner.max_concurrent_jobs
            );

            let notify = Arc::new(Notify::new());
            let mut q = self.inner.queue.lock().await;
            q.push_back((job_id.to_string(), Arc::clone(&notify)));
            let position = q.len();
            let estimated_wait_secs = (position as u64) * 15;

            AdmissionStatus::Queued {
                position,
                estimated_wait_secs,
                ticket: notify,
            }
        }
    }

    /// Wait until queue admits this job
    pub async fn wait_for_admission(&self, ticket: Arc<Notify>, timeout: Duration) -> Result<JobPermit, String> {
        tokio::select! {
            _ = ticket.notified() => {
                self.inner.active_jobs.fetch_add(1, Ordering::SeqCst);
                Ok(JobPermit {
                    guard: Arc::clone(&self.inner),
                })
            }
            _ = tokio::time::sleep(timeout) => {
                Err("Queue timeout: Memory did not free up within allotted threshold".to_string())
            }
        }
    }
}
