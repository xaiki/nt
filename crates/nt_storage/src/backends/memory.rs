use async_trait::async_trait;
use nt_core::{Article, Result, ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{StorageBackend, BackendConfig, EmbeddingModel};

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub collection: String,
    pub embedding_model: EmbeddingModel,
}

impl MemoryConfig {
    pub fn new() -> Self {
        Self {
            collection: "articles".to_string(),
            embedding_model: EmbeddingModel::default(),
        }
    }
}

impl BackendConfig for MemoryConfig {
    fn get_url(&self) -> String {
        "memory://".to_string()
    }

    fn get_collection(&self) -> String {
        self.collection.clone()
    }

    fn get_embedding_model(&self) -> EmbeddingModel {
        self.embedding_model.clone()
    }
}

pub struct MemoryStore {
    collection: String,
    articles: Vec<(Article, Vec<f32>)>,
}

impl MemoryStore {
    pub fn new(collection: String) -> Self {
        Self {
            collection,
            articles: Vec::new(),
        }
    }

    pub async fn store_article(&mut self, article: &Article, embedding: &[f32]) -> Result<()> {
        if let Some((existing_article, _)) = self.articles.iter_mut().find(|(a, _)| a.url == article.url) {
            *existing_article = article.clone();
        } else {
            self.articles.push((article.clone(), embedding.to_vec()));
        }
        Ok(())
    }

    pub async fn find_similar(&self, _embedding: &[f32], limit: usize) -> Result<Vec<Article>> {
        // For now, just return the most recent articles
        // In a real implementation, we would compute cosine similarity between embeddings
        let mut articles = self.articles.iter()
            .map(|(article, _)| article.clone())
            .collect::<Vec<_>>();
        articles.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        Ok(articles.into_iter().take(limit).collect())
    }

    pub async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        Ok(self.articles.iter()
            .filter(|(article, _)| article.source == source)
            .map(|(article, _)| article.clone())
            .collect())
    }
}

pub struct MemoryStorage {
    store: Arc<RwLock<MemoryStore>>,
}

impl MemoryStorage {
    pub async fn new() -> Result<Self> {
        let config = MemoryConfig::new();
        let store = Arc::new(RwLock::new(MemoryStore::new(
            config.get_collection(),
        )));
        Ok(Self { store })
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
        };

        let storage = MemoryStorage::new().await.unwrap();
        let embedding = vec![0.0; 384];
        storage.store_article(&article, &embedding).await.unwrap();
        let similar = storage.find_similar(&embedding, 1).await.unwrap();
        assert!(!similar.is_empty());
    }
} 