use async_trait::async_trait;
use nt_core::{Article, Result, ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{StorageBackend, BackendConfig, EmbeddingModel, UrlConfig};
use std::ops::Deref;

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
    r#"
    CREATE TABLE IF NOT EXISTS embeddings (
        url TEXT PRIMARY KEY,
        embedding BLOB NOT NULL,
        FOREIGN KEY (url) REFERENCES articles(url) ON DELETE CASCADE
    )
    "#,
    // Add future migrations here
];

#[derive(Debug, Clone)]
pub struct SQLiteConfig {
    pub config: BackendConfig,
}

impl SQLiteConfig {
    pub fn new() -> Self {
        let path = std::env::current_dir()
            .map(|p| p.join("articles.db"))
            .map(|p| p.to_str().unwrap_or("articles.db").to_string())
            .unwrap_or_else(|_| "articles.db".to_string());
        Self {
            config: BackendConfig::new(
                path,
                "articles".to_string(),
                EmbeddingModel::default(),
                1536,
            ),
        }
    }
}

impl Deref for SQLiteConfig {
    type Target = BackendConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

pub struct SQLiteStore {
    path: String,
    table: String,
    vector_size: u64,
}

impl SQLiteStore {
    pub fn new(path: String, table: String, vector_size: u64) -> Result<Self> {
        Ok(Self {
            path,
            table,
            vector_size,
        })
    }

    pub async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()> {
        // TODO: Implement SQLite storage
        Ok(())
    }

    pub async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Article>> {
        // TODO: Implement SQLite similarity search
        Ok(Vec::new())
    }

    pub async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        // TODO: Implement SQLite source filtering
        Ok(Vec::new())
    }
}

pub struct SQLiteStorage {
    store: Arc<RwLock<SQLiteStore>>,
    config: SQLiteConfig,
}

impl SQLiteStorage {
    pub async fn new() -> Result<Self> {
        let config = SQLiteConfig::new();
        let store = Arc::new(RwLock::new(SQLiteStore::new(
            config.url.clone(),
            config.collection.clone(),
            config.vector_size
        )?));
        Ok(Self { store, config })
    }
}

#[async_trait]
impl StorageBackend for SQLiteStorage {
    fn get_error_message() -> &'static str {
        "SQLite database should be accessible"
    }

    async fn new() -> Result<Self> where Self: Sized {
        Self::new().await
    }

    fn get_config(&mut self) -> Option<&mut BackendConfig> {
        Some(&mut self.config.config)
    }
}

#[async_trait]
impl ArticleStorage for SQLiteStorage {
    async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()> {
        let store = self.store.read().await;
        store.store_article(article, embedding).await
    }

    async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Article>> {
        let store = self.store.read().await;
        store.find_similar(embedding, limit).await
    }

    async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        let store = self.store.read().await;
        store.get_by_source(source).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_sqlite_storage() {
        let article = Article {
            url: "http://test.com".to_string(),
            title: "Test Article".to_string(),
            content: "This is a test article about politics.".to_string(),
            published_at: Utc::now(),
            source: "test".to_string(),
            sections: vec![],
            summary: None,
            authors: vec!["Test Author".to_string()],
        };

        let storage = SQLiteStorage::new().await.unwrap();
        let vector_size = storage.config.vector_size;
        let embedding = vec![0.0; vector_size as usize];
        storage.store_article(&article, &embedding).await.unwrap();
        let similar = storage.find_similar(&embedding, 1).await.unwrap();
        assert!(!similar.is_empty());
    }
} 