use async_trait::async_trait;
use nt_core::{Article, Result, ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{StorageBackend, BackendConfig, EmbeddingModel};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub config: BackendConfig,
}

impl MemoryConfig {
    pub fn new() -> Self {
        Self {
            config: BackendConfig::new(
                "memory://".to_string(),
                "articles".to_string(),
                EmbeddingModel::default(),
                768,
            ),
        }
    }
}

impl Deref for MemoryConfig {
    type Target = BackendConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

pub struct MemoryStore {
    collection: String,
    articles: std::collections::HashMap<String, (Article, Vec<f32>)>,
    vector_size: u64,
}

impl MemoryStore {
    pub fn new(collection: String, vector_size: u64) -> Self {
        Self {
            collection,
            articles: std::collections::HashMap::new(),
            vector_size,
        }
    }

    pub async fn store_article(&mut self, article: &Article, embedding: &[f32]) -> Result<()> {
        self.articles.insert(article.url.clone(), (article.clone(), embedding.to_vec()));
        Ok(())
    }

    pub async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Article>> {
        let mut articles_with_scores = Vec::new();

        for (_, (article, article_embedding)) in &self.articles {
            let similarity = nt_core::cosine_similarity(embedding, article_embedding);
            articles_with_scores.push((article.clone(), similarity));
        }

        articles_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(articles_with_scores.into_iter().take(limit).map(|(a, _)| a).collect())
    }

    pub async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        Ok(self.articles.values()
            .filter(|(article, _)| article.source == source)
            .map(|(article, _)| article.clone())
            .collect())
    }

    pub async fn delete_article(&mut self, url: &str) -> Result<()> {
        self.articles.remove(url);
        Ok(())
    }

    pub async fn get_article_embedding(&self, url: &str) -> Result<Vec<f32>> {
        match self.articles.get(url) {
            Some((_, embedding)) => Ok(embedding.clone()),
            None => Err(nt_core::Error::Database(format!("Article not found: {}", url)))
        }
    }
}

pub struct MemoryStorage {
    store: Arc<RwLock<MemoryStore>>,
    config: MemoryConfig,
}

impl MemoryStorage {
    pub async fn new() -> Result<Self> {
        let config = MemoryConfig::new();
        let store = Arc::new(RwLock::new(MemoryStore::new(
            config.collection.clone(),
            config.vector_size
        )));
        Ok(Self { store, config })
    }
}

#[async_trait]
impl StorageBackend for MemoryStorage {
    fn get_error_message() -> &'static str {
        "Memory storage should be available"
    }

    async fn new() -> Result<Self> where Self: Sized {
        Self::new().await
    }

    fn get_config(&mut self) -> Option<&mut BackendConfig> {
        Some(&mut self.config.config)
    }
}

#[async_trait]
impl ArticleStorage for MemoryStorage {
    async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()> {
        let mut store = self.store.write().await;
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
        let mut store = self.store.write().await;
        store.delete_article(url).await
    }

    async fn get_article_embedding(&self, url: &str) -> Result<Vec<f32>> {
        let store = self.store.read().await;
        store.get_article_embedding(url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_memory_storage() {
        let article = Article {
            url: "http://test.com".to_string(),
            title: "Test Article".to_string(),
            content: "This is a test article about politics.".to_string(),
            published_at: Utc::now(),
            source: "test".to_string(),
            sections: vec![],
            summary: None,
            authors: vec!["Test Author".to_string()],
            related_articles: Vec::new(),
        };

        let storage = MemoryStorage::new().await.unwrap();
        let vector_size = storage.config.vector_size;
        let embedding = vec![0.0; vector_size as usize];
        storage.store_article(&article, &embedding).await.unwrap();
        let similar = storage.find_similar(&embedding, 1).await.unwrap();
        assert!(!similar.is_empty());
    }
} 