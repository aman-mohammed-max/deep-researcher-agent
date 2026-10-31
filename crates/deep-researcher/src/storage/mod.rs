use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use tracing::info;

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

#[derive(Clone)]
pub struct Storage {
    pool: Pool<Sqlite>,
}

impl Storage {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let storage = Self { pool };
        storage.init_tables().await?;
        Ok(storage)
    }

    async fn init_tables(&self) -> Result<(), sqlx::Error> {
        info!("Initializing SQLite storage tables...");

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

            CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                key_prefix TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                is_revoked BOOLEAN NOT NULL DEFAULT 0
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_job(&self, id: &str, query: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO jobs (id, status, query, created_at, updated_at) VALUES (?, 'QUEUED', ?, ?, ?)",
        )
        .bind(id)
        .bind(query)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_job_status(&self, id: &str, status: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE jobs SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
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
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn fail_job(&self, id: &str, error: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE jobs SET status = 'FAILED', error_message = ?, updated_at = ? WHERE id = ?")
            .bind(error)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_job(&self, id: &str) -> Result<Option<JobRecord>, sqlx::Error> {
        sqlx::query_as::<_, JobRecord>("SELECT * FROM jobs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_jobs(&self, limit: i64) -> Result<Vec<JobRecord>, sqlx::Error> {
        sqlx::query_as::<_, JobRecord>("SELECT * FROM jobs ORDER BY updated_at DESC LIMIT ?")
            .bind(limit)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn add_event(&self, job_id: &str, event_type: &str, content: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO job_events (job_id, event_type, content, timestamp) VALUES (?, ?, ?, ?)")
            .bind(job_id)
            .bind(event_type)
            .bind(content)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_events(&self, job_id: &str) -> Result<Vec<JobEventRecord>, sqlx::Error> {
        sqlx::query_as::<_, JobEventRecord>("SELECT * FROM job_events WHERE job_id = ? ORDER BY id ASC")
            .bind(job_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn create_api_key(&self, name: &str) -> Result<String, sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let key = format!("dr_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
        let prefix = &key[0..7];
        let now = Utc::now().to_rfc3339();

        sqlx::query("INSERT INTO api_keys (id, key_prefix, name, created_at, is_revoked) VALUES (?, ?, ?, ?, 0)")
            .bind(&id)
            .bind(prefix)
            .bind(name)
            .bind(&now)
            .execute(&self.pool)
            .await?;

        Ok(key)
    }

    pub async fn verify_api_key(&self, key: &str) -> Result<bool, sqlx::Error> {
        if key.len() < 7 {
            return Ok(false);
        }
        let prefix = &key[0..7];
        let record = sqlx::query_as::<_, ApiKeyRecord>(
            "SELECT * FROM api_keys WHERE key_prefix = ? AND is_revoked = 0",
        )
        .bind(prefix)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.is_some())
    }

    pub async fn list_api_keys(&self) -> Result<Vec<ApiKeyRecord>, sqlx::Error> {
        sqlx::query_as::<_, ApiKeyRecord>("SELECT * FROM api_keys ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn revoke_api_key(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_keys SET is_revoked = 1 WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
