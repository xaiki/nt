use async_trait::async_trait;
use nt_core::{Article, Result, storage::ArticleStorage};
use std::sync::Arc;
use sqlx::{sqlite::SqlitePool, Row};
use serde_json::Value;
use crate::StorageBackend;
use std::path::PathBuf;
use anyhow::anyhow;

const MIGRATIONS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS articles (
        url TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        source TEXT NOT NULL,
        published_at TEXT NOT NULL,
        sections TEXT,
        summary TEXT
    )
    "#,
    // Add future migrations here
];

pub struct SQLiteStorage {
    pool: Arc<SqlitePool>,
    db_path: PathBuf,
}

impl StorageBackend for SQLiteStorage {
    fn get_error_message() -> &'static str {
        "SQLite database should be available at ./articles.db"
    }

    async fn new() -> Result<Self> {
        let db_path = std::env::current_dir()
            .map_err(|e| nt_core::Error::External(anyhow!("Failed to get current directory: {}", e)))?
            .join("articles.db");
        Self::new_with_path(&db_path).await
    }
}

impl SQLiteStorage {
    pub async fn new_with_path(db_path: &PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| nt_core::Error::External(anyhow!("Failed to create database directory: {}", e)))?;
        }

        // Create empty database file if it doesn't exist
        if !db_path.exists() {
            std::fs::File::create(db_path)
                .map_err(|e| nt_core::Error::External(anyhow!("Failed to create database file: {}", e)))?;
        }

        let db_path_str = db_path.to_str()
            .ok_or_else(|| nt_core::Error::External(anyhow!("Invalid database path")))?;
        tracing::info!("Attempting to connect to SQLite database at: {}", db_path_str);

        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path_str))
            .await
            .map_err(|e| nt_core::Error::External(anyhow!("Failed to connect to database: {}", e)))?;

        tracing::info!("Successfully connected to SQLite database");

        // Run migrations
        for (i, migration) in MIGRATIONS.iter().enumerate() {
            sqlx::query(migration)
                .execute(&pool)
                .await
                .map_err(|e| nt_core::Error::External(anyhow!("Failed to run migration {}: {}", i, e)))?;
        }

        Ok(Self {
            pool: Arc::new(pool),
            db_path: db_path.clone(),
        })
    }

    pub fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }
}

#[async_trait]
impl ArticleStorage for SQLiteStorage {
    async fn store_article(&self, article: &Article) -> Result<()> {
        let sections = serde_json::to_string(&article.sections)
            .map_err(|e| nt_core::Error::Serialization(e))?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO articles 
            (url, title, content, source, published_at, sections, summary)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&article.url)
        .bind(&article.title)
        .bind(&article.content)
        .bind(&article.source)
        .bind(article.published_at.to_rfc3339())
        .bind(sections)
        .bind(article.summary.as_deref())
        .execute(&*self.pool)
        .await
        .map_err(|e| nt_core::Error::External(anyhow!("Failed to store article: {}", e)))?;

        Ok(())
    }

    async fn find_similar(&self, article: &Article, limit: usize) -> Result<Vec<Article>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM articles 
            WHERE source = ? AND url != ?
            ORDER BY published_at DESC
            LIMIT ?
            "#,
        )
        .bind(&article.source)
        .bind(&article.url)
        .bind(limit as i64)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| nt_core::Error::External(anyhow!("Failed to find similar articles: {}", e)))?;

        let mut articles = Vec::new();
        for row in rows {
            let sections: String = row.get("sections");
            let sections: Vec<Value> = serde_json::from_str(&sections)
                .map_err(|e| nt_core::Error::Serialization(e))?;

            articles.push(Article {
                url: row.get("url"),
                title: row.get("title"),
                content: row.get("content"),
                source: row.get("source"),
                published_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("published_at"))
                    .map_err(|e| nt_core::Error::External(anyhow!("Failed to parse date: {}", e)))?
                    .with_timezone(&chrono::Utc),
                sections: sections.into_iter()
                    .filter_map(|v| serde_json::from_value(v).ok())
                    .collect(),
                summary: row.get::<Option<String>, _>("summary"),
            });
        }

        Ok(articles)
    }

    async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM articles 
            WHERE source = ?
            ORDER BY published_at DESC
            "#,
        )
        .bind(source)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| nt_core::Error::External(anyhow!("Failed to get articles by source: {}", e)))?;

        let mut articles = Vec::new();
        for row in rows {
            let sections: String = row.get("sections");
            let sections: Vec<Value> = serde_json::from_str(&sections)
                .map_err(|e| nt_core::Error::Serialization(e))?;

            articles.push(Article {
                url: row.get("url"),
                title: row.get("title"),
                content: row.get("content"),
                source: row.get("source"),
                published_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("published_at"))
                    .map_err(|e| nt_core::Error::External(anyhow!("Failed to parse date: {}", e)))?
                    .with_timezone(&chrono::Utc),
                sections: sections.into_iter()
                    .filter_map(|v| serde_json::from_value(v).ok())
                    .collect(),
                summary: row.get::<Option<String>, _>("summary"),
            });
        }

        Ok(articles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_sqlite_storage() {
        // Create a temporary directory for the test database
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let storage = SQLiteStorage::new_with_path(&db_path).await.unwrap();
        let article = Article {
            url: "http://example.com".to_string(),
            title: "Test Article".to_string(),
            content: "Test content".to_string(),
            published_at: chrono::Utc::now(),
            source: "test".to_string(),
            sections: vec![],
            summary: None,
        };

        storage.store_article(&article).await.unwrap();
        let similar = storage.find_similar(&article, 1).await.unwrap();
        assert!(similar.is_empty());

        // Test database will be automatically cleaned up when temp_dir is dropped
    }
} 