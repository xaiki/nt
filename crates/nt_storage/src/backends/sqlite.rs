use async_trait::async_trait;
use nt_core::{Article, Result, ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{StorageBackend, BackendConfig, EmbeddingModel, UrlConfig};
use std::ops::Deref;
use sqlx::{sqlite::SqlitePool, Row};
use serde_json::Value;

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
        summary TEXT,
        authors TEXT NOT NULL DEFAULT '[]',
        related_articles TEXT NOT NULL DEFAULT '[]'
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS embeddings (
        url TEXT PRIMARY KEY,
        embedding BLOB NOT NULL,
        FOREIGN KEY (url) REFERENCES articles(url) ON DELETE CASCADE
    )
    "#,
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
    pool: SqlitePool,
    table: String,
    vector_size: u64,
}

impl SQLiteStore {
    pub async fn new(path: String, table: String, vector_size: u64) -> Result<Self> {
        // Create SQLite connection pool
        let pool = SqlitePool::connect(&format!("sqlite:{}", path))
            .await
            .map_err(|e| nt_core::Error::Database(format!("Failed to connect to SQLite: {}", e)))?;

        // Apply migrations
        for (i, migration) in MIGRATIONS.iter().enumerate() {
            sqlx::query(migration)
                .execute(&pool)
                .await
                .map_err(|e| nt_core::Error::Database(format!("Failed to apply migration {}: {}", i, e)))?;
        }

        Ok(Self {
            pool,
            table,
            vector_size,
        })
    }

    pub async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()> {
        tracing::debug!("💾 Storing article: {}", article.title);
        tracing::debug!("🔗 URL: {}", article.url);
        tracing::debug!("📝 Content length: {} chars", article.content.len());
        tracing::debug!("👥 Authors: {:?}", article.authors);
        tracing::debug!("📅 Published at: {}", article.published_at);
        tracing::debug!("🔢 Embedding size: {}", embedding.len());
        tracing::debug!("🔗 Related articles: {}", article.related_articles.len());

        let sections_json = match serde_json::to_string(&article.sections) {
            Ok(json) => {
                tracing::debug!("✅ Successfully serialized sections");
                json
            }
            Err(e) => {
                tracing::error!("❌ Failed to serialize sections: {}", e);
                return Err(nt_core::Error::Serialization(e));
            }
        };

        let authors_json = match serde_json::to_string(&article.authors) {
            Ok(json) => {
                tracing::debug!("✅ Successfully serialized authors");
                json
            }
            Err(e) => {
                tracing::error!("❌ Failed to serialize authors: {}", e);
                return Err(nt_core::Error::Serialization(e));
            }
        };

        let related_articles_json = match serde_json::to_string(&article.related_articles) {
            Ok(json) => {
                tracing::debug!("✅ Successfully serialized related articles");
                json
            }
            Err(e) => {
                tracing::error!("❌ Failed to serialize related articles: {}", e);
                return Err(nt_core::Error::Serialization(e));
            }
        };

        // Store article
        tracing::debug!("📝 Inserting article into database");
        match sqlx::query(
            r#"
            INSERT OR REPLACE INTO articles (
                url, title, content, source, published_at, sections, summary, authors, related_articles
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&article.url)
        .bind(&article.title)
        .bind(&article.content)
        .bind(&article.source)
        .bind(article.published_at.to_rfc3339())
        .bind(&sections_json)
        .bind(article.summary.as_deref())
        .bind(&authors_json)
        .bind(&related_articles_json)
        .execute(&self.pool)
        .await {
            Ok(_) => tracing::debug!("✅ Successfully stored article"),
            Err(e) => {
                tracing::error!("❌ Failed to store article: {}", e);
                return Err(nt_core::Error::Database(format!("Failed to store article: {}", e)));
            }
        }

        // Store embedding
        tracing::debug!("🔢 Converting embedding to bytes");
        let embedding_bytes = unsafe {
            std::slice::from_raw_parts(
                embedding.as_ptr() as *const u8,
                embedding.len() * std::mem::size_of::<f32>(),
            )
        };

        tracing::debug!("💾 Storing embedding in database");
        match sqlx::query(
            r#"
            INSERT OR REPLACE INTO embeddings (url, embedding) VALUES (?, ?)
            "#,
        )
        .bind(&article.url)
        .bind(embedding_bytes)
        .execute(&self.pool)
        .await {
            Ok(_) => tracing::debug!("✅ Successfully stored embedding"),
            Err(e) => {
                tracing::error!("❌ Failed to store embedding: {}", e);
                return Err(nt_core::Error::Database(format!("Failed to store embedding: {}", e)));
            }
        }

        tracing::debug!("✨ Article and embedding stored successfully");
        Ok(())
    }

    pub async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Article>> {
        tracing::debug!("🔍 Finding similar articles with embedding size: {}", embedding.len());
        
        // For SQLite, we'll use a simple cosine similarity calculation
        let _embedding_bytes = unsafe {
            std::slice::from_raw_parts(
                embedding.as_ptr() as *const u8,
                embedding.len() * std::mem::size_of::<f32>(),
            )
        };

        // Get all articles with their embeddings
        tracing::debug!("📚 Fetching all articles with embeddings");
        let rows = match sqlx::query(
            r#"
            SELECT a.*, e.embedding
            FROM articles a
            JOIN embeddings e ON a.url = e.url
            "#,
        )
        .fetch_all(&self.pool)
        .await {
            Ok(rows) => {
                tracing::debug!("✅ Successfully fetched {} articles", rows.len());
                rows
            }
            Err(e) => {
                tracing::error!("❌ Failed to fetch articles: {}", e);
                return Err(nt_core::Error::Database(format!("Failed to fetch articles: {}", e)));
            }
        };

        let mut articles_with_scores = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            tracing::debug!("📄 Processing article {}/{}", i + 1, rows.len());
            
            let article = match self.row_to_article(row) {
                Ok(article) => {
                    tracing::debug!("✅ Successfully converted row to article: {}", article.title);
                    article
                }
                Err(e) => {
                    tracing::error!("❌ Failed to convert row to article: {}", e);
                    continue;
                }
            };

            let embedding_bytes: Vec<u8> = row.get("embedding");
            tracing::debug!("🔢 Got embedding bytes of size: {}", embedding_bytes.len());
            
            let article_embedding = unsafe {
                std::slice::from_raw_parts(
                    embedding_bytes.as_ptr() as *const f32,
                    embedding_bytes.len() / std::mem::size_of::<f32>(),
                )
            };
            
            tracing::debug!("🔢 Converted to f32 slice of size: {}", article_embedding.len());
            
            let similarity = nt_core::cosine_similarity(embedding, article_embedding);
            tracing::debug!("📊 Calculated similarity score: {}", similarity);
            
            articles_with_scores.push((article, similarity));
        }

        // Sort by similarity and take top N
        tracing::debug!("📊 Sorting {} articles by similarity", articles_with_scores.len());
        articles_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let result: Vec<Article> = articles_with_scores.into_iter().take(limit).map(|(a, _)| a).collect();
        tracing::debug!("✨ Returning {} similar articles", result.len());
        Ok(result)
    }

    pub async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM articles WHERE source = ?
            "#,
        )
        .bind(source)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| nt_core::Error::Database(format!("Failed to fetch articles: {}", e)))?;

        rows.into_iter()
            .map(|row| self.row_to_article(&row))
            .collect()
    }

    fn row_to_article(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Article> {
        tracing::debug!("🔍 Converting SQLite row to Article");
        
        let sections_json: Option<String> = row.get("sections");
        tracing::debug!("📄 Sections JSON: {:?}", sections_json);
        
        let authors_json: String = row.get("authors");
        tracing::debug!("👥 Authors JSON: {}", authors_json);
        
        let related_articles_json: Option<String> = row.get("related_articles");
        tracing::debug!("🔗 Related articles JSON: {:?}", related_articles_json);

        let sections: Vec<nt_core::ArticleSection> = if let Some(json) = sections_json {
            tracing::debug!("📝 Deserializing sections from JSON");
            match serde_json::from_str::<Vec<nt_core::ArticleSection>>(&json) {
                Ok(sections) => {
                    tracing::debug!("✅ Successfully deserialized {} sections", sections.len());
                    sections
                }
                Err(e) => {
                    tracing::error!("❌ Failed to deserialize sections: {}", e);
                    return Err(nt_core::Error::Serialization(e));
                }
            }
        } else {
            tracing::debug!("📝 No sections found, using empty vector");
            Vec::new()
        };

        tracing::debug!("👥 Deserializing authors from JSON");
        let authors: Vec<String> = match serde_json::from_str::<Vec<String>>(&authors_json) {
            Ok(authors) => {
                tracing::debug!("✅ Successfully deserialized {} authors", authors.len());
                authors
            }
            Err(e) => {
                tracing::error!("❌ Failed to deserialize authors: {}", e);
                return Err(nt_core::Error::Serialization(e));
            }
        };

        let related_articles: Vec<nt_core::RelatedArticle> = if let Some(json) = related_articles_json {
            tracing::debug!("🔗 Deserializing related articles from JSON");
            match serde_json::from_str::<Vec<nt_core::RelatedArticle>>(&json) {
                Ok(articles) => {
                    tracing::debug!("✅ Successfully deserialized {} related articles", articles.len());
                    articles
                }
                Err(e) => {
                    tracing::error!("❌ Failed to deserialize related articles: {}", e);
                    tracing::error!("📄 JSON content: {}", json);
                    // If deserialization fails, try to parse as a raw JSON array and filter out invalid entries
                    match serde_json::from_str::<Vec<serde_json::Value>>(&json) {
                        Ok(values) => {
                            let mut valid_articles = Vec::new();
                            for value in values {
                                if let Ok(article) = serde_json::from_value(value) {
                                    valid_articles.push(article);
                                }
                            }
                            tracing::debug!("✅ Successfully recovered {} valid related articles", valid_articles.len());
                            valid_articles
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to parse as raw JSON array: {}", e);
                            Vec::new()
                        }
                    }
                }
            }
        } else {
            tracing::debug!("🔗 No related articles found, using empty vector");
            Vec::new()
        };

        let published_at_str: String = row.get("published_at");
        tracing::debug!("📅 Parsing published_at: {}", published_at_str);
        
        let published_at = match chrono::DateTime::parse_from_rfc3339(&published_at_str) {
            Ok(dt) => {
                tracing::debug!("✅ Successfully parsed date");
                dt.with_timezone(&chrono::Utc)
            }
            Err(e) => {
                tracing::error!("❌ Failed to parse date: {}", e);
                return Err(nt_core::Error::Database(format!("Failed to parse date: {}", e)));
            }
        };

        tracing::debug!("✨ Successfully converted row to Article");
        Ok(Article {
            url: row.get("url"),
            title: row.get("title"),
            content: row.get("content"),
            source: row.get("source"),
            published_at,
            sections,
            summary: row.get("summary"),
            authors,
            related_articles,
        })
    }

    pub async fn delete_article(&self, url: &str) -> Result<()> {
        tracing::debug!("🗑️ Deleting article: {}", url);
        
        // The embedding will be automatically deleted due to the ON DELETE CASCADE
        match sqlx::query("DELETE FROM articles WHERE url = ?")
            .bind(url)
            .execute(&self.pool)
            .await {
                Ok(_) => {
                    tracing::debug!("✅ Successfully deleted article");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("❌ Failed to delete article: {}", e);
                    Err(nt_core::Error::Database(format!("Failed to delete article: {}", e)))
                }
            }
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
        ).await?));
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

    async fn delete_article(&self, url: &str) -> Result<()> {
        let store = self.store.read().await;
        store.delete_article(url).await
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
            related_articles: vec![],
        };

        let storage = SQLiteStorage::new().await.unwrap();
        let vector_size = storage.config.vector_size;
        let embedding = vec![0.0; vector_size as usize];
        storage.store_article(&article, &embedding).await.unwrap();
        let similar = storage.find_similar(&embedding, 1).await.unwrap();
        assert!(!similar.is_empty());
    }
} 