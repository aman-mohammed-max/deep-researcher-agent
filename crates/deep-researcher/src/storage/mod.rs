// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{
    postgres::PgPoolOptions,
    sqlite::SqlitePoolOptions,
    Pool, Postgres, Sqlite,
};
use tracing::info;

pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobRecord {
    pub id: String,
    pub status: String,
    pub query: String,
    pub report: Option<String>,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobEventRecord {
    pub id: i64,
    pub job_id: String,
    pub event_type: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKeyRecord {
    pub id: String,
    pub key_prefix: String,
    pub name: String,
    pub created_at: String,
    pub is_revoked: bool,
}

/// Persistent Storage Backend supporting either PostgreSQL or persistent SQLite
#[derive(Clone)]
pub enum PersistentBackend {
    Sqlite(Pool<Sqlite>),
    Postgres(Pool<Postgres>),
}

/// Tiered Storage Engine:
/// - `runtime_pool`: Ephemeral / RAM-like SQLite for active research workflows, streaming events & tasks.
/// - `persistent_backend`: Persistent SQL (PostgreSQL or persistent SQLite) for API keys and durable data.
#[derive(Clone)]
pub struct Storage {
    runtime_pool: Pool<Sqlite>,
    persistent_backend: PersistentBackend,
    runtime_url: String,
}

impl Storage {
    /// Initialize tiered dual-storage with separate ephemeral runtime and persistent database URLs.
    ///
    /// - `runtime_url`: e.g. "sqlite::memory:?cache=shared" or local SQLite for ephemeral research jobs.
    /// - `persistent_url`: e.g. "postgres://user:pass@host:5432/db" or "sqlite:deep_researcher_keys.db?mode=rwc".
    pub async fn new(runtime_url: &str, persistent_url: &str) -> Result<Self, sqlx::Error> {
        info!("Initializing Tiered Storage Engine...");
        info!("  Runtime DB (RAM/Ephemeral): {}", runtime_url);

        // 1. Initialize Runtime SQLite Pool (Jobs, Streaming Events, Working Memory)
        let runtime_pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(runtime_url)
            .await?;

        // 2. Initialize Persistent Backend (PostgreSQL or SQLite for API Keys)
        let persistent_backend = if persistent_url.starts_with("postgres://")
            || persistent_url.starts_with("postgresql://")
        {
            info!("  Persistent DB: PostgreSQL [{}]", persistent_url);
            let pg_pool = PgPoolOptions::new()
                .max_connections(10)
                .acquire_timeout(std::time::Duration::from_secs(3))
                .connect(persistent_url)
                .await?;
            PersistentBackend::Postgres(pg_pool)
        } else {
            info!("  Persistent DB: SQLite [{}]", persistent_url);
            let sqlite_pool = SqlitePoolOptions::new()
                .max_connections(5)
                .connect(persistent_url)
                .await?;
            PersistentBackend::Sqlite(sqlite_pool)
        };

        let storage = Self {
            runtime_pool,
            persistent_backend,
            runtime_url: runtime_url.to_string(),
        };

        storage.init_tables().await?;
        Ok(storage)
    }

    /// Single URL convenience constructor (used in unit tests and default standalone CLI).
    pub async fn from_url(database_url: &str) -> Result<Self, sqlx::Error> {
        Self::new(database_url, database_url).await
    }

    /// Report persistent store driver type ("PostgreSQL" or "SQLite").
    pub fn persistent_type(&self) -> &'static str {
        match self.persistent_backend {
            PersistentBackend::Postgres(_) => "PostgreSQL",
            PersistentBackend::Sqlite(_) => "SQLite",
        }
    }

    /// Whether the runtime storage is running in volatile RAM (in-memory).
    pub fn is_in_memory_runtime(&self) -> bool {
        self.runtime_url.contains(":memory:")
    }

    /// Initialize tables across both runtime and persistent database tiers.
    async fn init_tables(&self) -> Result<(), sqlx::Error> {
        // 1. Runtime Tables: jobs & job_events (in-memory / ephemeral SQLite)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                query TEXT NOT NULL,
                report TEXT,
                prompt_tokens INTEGER NOT NULL DEFAULT 0,
                completion_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                error_message TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS job_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.runtime_pool)
        .await?;

        // 2. Persistent Table: api_keys (PostgreSQL or persistent SQLite)
        match &self.persistent_backend {
            PersistentBackend::Postgres(pg_pool) => {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS api_keys (
                        id VARCHAR(64) PRIMARY KEY,
                        key_prefix VARCHAR(32) NOT NULL,
                        key_hash VARCHAR(128),
                        name VARCHAR(255) NOT NULL,
                        created_at VARCHAR(64) NOT NULL,
                        is_revoked BOOLEAN NOT NULL DEFAULT FALSE
                    );
                    "#,
                )
                .execute(pg_pool)
                .await?;

                let _ = sqlx::query("ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS key_hash VARCHAR(128)")
                    .execute(pg_pool)
                    .await;
            }
            PersistentBackend::Sqlite(sqlite_pool) => {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS api_keys (
                        id TEXT PRIMARY KEY,
                        key_prefix TEXT NOT NULL,
                        key_hash TEXT,
                        name TEXT NOT NULL,
                        created_at TEXT NOT NULL,
                        is_revoked BOOLEAN NOT NULL DEFAULT 0
                    );
                    "#,
                )
                .execute(sqlite_pool)
                .await?;

                let _ = sqlx::query("ALTER TABLE api_keys ADD COLUMN key_hash TEXT")
                    .execute(sqlite_pool)
                    .await;
            }
        }

        Ok(())
    }

    // =========================================================================
    // EPHEMERAL RUNTIME (RAM / JOBS / EVENTS)
    // =========================================================================

    pub async fn create_job(&self, id: &str, query: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO jobs (id, status, query, created_at, updated_at) VALUES (?, 'QUEUED', ?, ?, ?)",
        )
        .bind(id)
        .bind(query)
        .bind(&now)
        .bind(&now)
        .execute(&self.runtime_pool)
        .await?;
        Ok(())
    }

    pub async fn update_job_status(&self, id: &str, status: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE jobs SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(&self.runtime_pool)
            .await?;
        Ok(())
    }

    pub async fn complete_job(
        &self,
        id: &str,
        report: &str,
        prompt_tokens: u32,
        completion_tokens: u32,
        total_tokens: u32,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE jobs 
            SET status = 'COMPLETED', report = ?, prompt_tokens = ?, completion_tokens = ?, total_tokens = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(report)
        .bind(prompt_tokens as i64)
        .bind(completion_tokens as i64)
        .bind(total_tokens as i64)
        .bind(&now)
        .bind(id)
        .execute(&self.runtime_pool)
        .await?;
        Ok(())
    }

    pub async fn fail_job(&self, id: &str, error: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE jobs SET status = 'FAILED', error_message = ?, updated_at = ? WHERE id = ?")
            .bind(error)
            .bind(&now)
            .bind(id)
            .execute(&self.runtime_pool)
            .await?;
        Ok(())
    }

    pub async fn get_job(&self, id: &str) -> Result<Option<JobRecord>, sqlx::Error> {
        sqlx::query_as::<_, JobRecord>("SELECT * FROM jobs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.runtime_pool)
            .await
    }

    pub async fn list_jobs(&self, limit: i64) -> Result<Vec<JobRecord>, sqlx::Error> {
        sqlx::query_as::<_, JobRecord>("SELECT * FROM jobs ORDER BY updated_at DESC LIMIT ?")
            .bind(limit)
            .fetch_all(&self.runtime_pool)
            .await
    }

    pub async fn add_event(&self, job_id: &str, event_type: &str, content: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO job_events (job_id, event_type, content, timestamp) VALUES (?, ?, ?, ?)")
            .bind(job_id)
            .bind(event_type)
            .bind(content)
            .bind(&now)
            .execute(&self.runtime_pool)
            .await?;
        Ok(())
    }

    pub async fn get_events(&self, job_id: &str) -> Result<Vec<JobEventRecord>, sqlx::Error> {
        sqlx::query_as::<_, JobEventRecord>("SELECT * FROM job_events WHERE job_id = ? ORDER BY id ASC")
            .bind(job_id)
            .fetch_all(&self.runtime_pool)
            .await
    }

    // =========================================================================
    // PERSISTENT STORAGE (POSTGRESQL OR PERSISTENT SQLITE)
    // =========================================================================

    pub async fn create_api_key(&self, name: &str) -> Result<String, sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let key = format!("dr_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
        let prefix = &key[0..7];
        let key_hash = hash_api_key(&key);
        let now = Utc::now().to_rfc3339();

        match &self.persistent_backend {
            PersistentBackend::Postgres(pg_pool) => {
                sqlx::query(
                    "INSERT INTO api_keys (id, key_prefix, key_hash, name, created_at, is_revoked) VALUES ($1, $2, $3, $4, $5, FALSE)",
                )
                .bind(&id)
                .bind(prefix)
                .bind(&key_hash)
                .bind(name)
                .bind(&now)
                .execute(pg_pool)
                .await?;
            }
            PersistentBackend::Sqlite(sqlite_pool) => {
                sqlx::query(
                    "INSERT INTO api_keys (id, key_prefix, key_hash, name, created_at, is_revoked) VALUES (?, ?, ?, ?, ?, 0)",
                )
                .bind(&id)
                .bind(prefix)
                .bind(&key_hash)
                .bind(name)
                .bind(&now)
                .execute(sqlite_pool)
                .await?;
            }
        }

        Ok(key)
    }

    pub async fn verify_api_key(&self, key: &str) -> Result<bool, sqlx::Error> {
        let key = key.trim();
        if key.is_empty() {
            return Ok(false);
        }

        // 1. Master environment tokens
        if let Ok(admin_key) = std::env::var("ADMIN_API_KEY") {
            if !admin_key.is_empty() && key == admin_key.trim() {
                return Ok(true);
            }
        }
        if let Ok(auth_token) = std::env::var("AUTH_TOKEN") {
            if !auth_token.is_empty() && key == auth_token.trim() {
                return Ok(true);
            }
        }

        let prefix = if key.len() >= 7 { &key[0..7] } else { key };
        let key_hash = hash_api_key(key);

        let record = match &self.persistent_backend {
            PersistentBackend::Postgres(pg_pool) => {
                sqlx::query_as::<_, ApiKeyRecord>(
                    "SELECT id, key_prefix, name, created_at, is_revoked FROM api_keys WHERE (key_hash = $1 OR key_prefix = $2) AND is_revoked = FALSE",
                )
                .bind(&key_hash)
                .bind(prefix)
                .fetch_optional(pg_pool)
                .await?
            }
            PersistentBackend::Sqlite(sqlite_pool) => {
                sqlx::query_as::<_, ApiKeyRecord>(
                    "SELECT id, key_prefix, name, created_at, is_revoked FROM api_keys WHERE (key_hash = ? OR key_prefix = ?) AND is_revoked = 0",
                )
                .bind(&key_hash)
                .bind(prefix)
                .fetch_optional(sqlite_pool)
                .await?
            }
        };

        Ok(record.is_some())
    }

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKeyRecord>, sqlx::Error> {
        match &self.persistent_backend {
            PersistentBackend::Postgres(pg_pool) => {
                sqlx::query_as::<_, ApiKeyRecord>(
                    "SELECT id, key_prefix, name, created_at, is_revoked FROM api_keys ORDER BY created_at DESC",
                )
                .fetch_all(pg_pool)
                .await
            }
            PersistentBackend::Sqlite(sqlite_pool) => {
                sqlx::query_as::<_, ApiKeyRecord>(
                    "SELECT id, key_prefix, name, created_at, is_revoked FROM api_keys ORDER BY created_at DESC",
                )
                .fetch_all(sqlite_pool)
                .await
            }
        }
    }

    pub async fn revoke_api_key(&self, id: &str) -> Result<(), sqlx::Error> {
        match &self.persistent_backend {
            PersistentBackend::Postgres(pg_pool) => {
                sqlx::query("UPDATE api_keys SET is_revoked = TRUE WHERE id = $1")
                    .bind(id)
                    .execute(pg_pool)
                    .await?;
            }
            PersistentBackend::Sqlite(sqlite_pool) => {
                sqlx::query("UPDATE api_keys SET is_revoked = 1 WHERE id = ?")
                    .bind(id)
                    .execute(sqlite_pool)
                    .await?;
            }
        }
        Ok(())
    }
}
