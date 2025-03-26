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
    CREATE TABLE IF NOT EXISTS migrations (
        id INTEGER PRIMARY KEY,
        applied_at TEXT NOT NULL
    );

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
    r#"
    ALTER TABLE articles ADD COLUMN authors TEXT NOT NULL DEFAULT '[]'
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
            // Check if migration has been applied
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='migrations'")
                .fetch_one(&pool)
                .await
                .map_err(|e| nt_core::Error::External(anyhow!("Failed to check migrations table: {}", e)))?;

            if count == 0 && i == 0 {
                // First migration, create migrations table
                sqlx::query(migration)
                    .execute(&pool)
                    .await
                    .map_err(|e| nt_core::Error::External(anyhow!("Failed to create migrations table: {}", e)))?;
                
                // Record first migration
                sqlx::query("INSERT INTO migrations (id, applied_at) VALUES (?, ?)")
                    .bind(i as i64)
                    .bind(chrono::Utc::now().to_rfc3339())
                    .execute(&pool)
                    .await
                    .map_err(|e| nt_core::Error::External(anyhow!("Failed to record migration: {}", e)))?;
                continue;
            }

            // Check if this migration has been applied
            let applied: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM migrations WHERE id = ?")
                .bind(i as i64)
                .fetch_one(&pool)
                .await
                .map_err(|e| nt_core::Error::External(anyhow!("Failed to check migration status: {}", e)))?;

            if applied == 0 {
                // Run the migration
                sqlx::query(migration)
                    .execute(&pool)
                    .await
                    .map_err(|e| nt_core::Error::External(anyhow!("Failed to run migration {}: {}", i, e)))?;

                // Record the migration
                sqlx::query("INSERT INTO migrations (id, applied_at) VALUES (?, ?)")
                    .bind(i as i64)
                    .bind(chrono::Utc::now().to_rfc3339())
                    .execute(&pool)
                    .await
                    .map_err(|e| nt_core::Error::External(anyhow!("Failed to record migration: {}", e)))?;
            }
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
        let authors = serde_json::to_string(&article.authors)
            .map_err(|e| nt_core::Error::Serialization(e))?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO articles 
            (url, title, content, source, published_at, sections, summary, authors)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&article.url)
        .bind(&article.title)
        .bind(&article.content)
        .bind(&article.source)
        .bind(article.published_at.to_rfc3339())
        .bind(sections)
        .bind(article.summary.as_deref())
        .bind(authors)
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
            let authors: String = row.get("authors");
            let authors: Vec<String> = serde_json::from_str(&authors)
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
                authors,
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
            let authors: String = row.get("authors");
            let authors: Vec<String> = serde_json::from_str(&authors)
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
                authors,
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
        let article = Article {
            url: "http://test.com".to_string(),
            title: "Test Article".to_string(),
            content: "This is a test article about politics.".to_string(),
            published_at: chrono::Utc::now(),
            source: "test".to_string(),
            sections: vec![],
            summary: None,
            authors: vec!["Test Author".to_string()],
        };

        let storage = SQLiteStorage::new().await.unwrap();
        storage.store_article(&article).await.unwrap();
        let similar = storage.find_similar(&article, 1).await.unwrap();
        assert!(!similar.is_empty());
    }
} 