use async_trait::async_trait;
use nt_core::{Article, Result, storage::ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::StorageBackend;

pub struct InMemoryStorage {
    articles: Arc<RwLock<HashMap<String, Article>>>,
}

impl StorageBackend for InMemoryStorage {
    fn get_error_message() -> &'static str {
        "In-memory storage initialization failed"
    }

    async fn new() -> Result<Self> {
        Ok(Self {
            articles: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

#[async_trait]
impl ArticleStorage for InMemoryStorage {
    async fn store_article(&self, article: &Article) -> Result<()> {
        let mut articles = self.articles.write().await;
        articles.insert(article.url.clone(), article.clone());
        Ok(())
    }

    async fn find_similar(&self, article: &Article, limit: usize) -> Result<Vec<Article>> {
        let articles = self.articles.read().await;
        let mut similar = Vec::new();
        
        // For now, just return articles from the same source
        for (_, a) in articles.iter() {
            if a.source == article.source && a.url != article.url {
                similar.push(a.clone());
                if similar.len() >= limit {
                    break;
                }
            }
        }
        
        Ok(similar)
    }

    async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        let articles = self.articles.read().await;
        Ok(articles
            .values()
            .filter(|a| a.source == source)
            .cloned()
            .collect())
    }
}

impl Clone for InMemoryStorage {
    fn clone(&self) -> Self {
        Self {
            articles: Arc::clone(&self.articles),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryStorage::new().await.unwrap();
        let article = Article {
            url: "http://example.com".to_string(),
            title: "Test Article".to_string(),
            content: "Test content".to_string(),
            published_at: chrono::Utc::now(),
            source: "test".to_string(),
            sections: vec![],
            summary: None,
            authors: vec!["Test Author".to_string()],
        };

        storage.store_article(&article).await.unwrap();
        let similar = storage.find_similar(&article, 1).await.unwrap();
        assert!(similar.is_empty());
    }
} 