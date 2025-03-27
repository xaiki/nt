use async_trait::async_trait;
use nt_core::{Article, Result, ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use chromadb::v1::{
    client::ChromaClient,
    collection::{CollectionEntries, QueryOptions},
};
use crate::{StorageBackend, BackendConfig, EmbeddingModel};
use std::fmt;
use std::env;

#[derive(Debug, Clone)]
pub struct ChromaConfig {
    pub url: String,
    pub collection: String,
    pub embedding_model: EmbeddingModel,
}

impl ChromaConfig {
    pub fn new() -> Self {
        let host = env::var("CHROMA_HOST").unwrap_or_else(|_| "chroma".to_string());
        let url = format!("http://{}:8000", host);
        Self {
            url,
            collection: "articles".to_string(),
            embedding_model: EmbeddingModel::default(),
        }
    }
}

impl BackendConfig for ChromaConfig {
    fn get_url(&self) -> String {
        self.url.clone()
    }

    fn get_collection(&self) -> String {
        self.collection.clone()
    }

    fn get_embedding_model(&self) -> EmbeddingModel {
        self.embedding_model.clone()
    }
}

pub struct EmbeddingStore {
    client: Arc<ChromaClient>,
    collection_name: String,
}

impl EmbeddingStore {
    pub fn new(collection_name: String) -> Result<Self> {
        let client = Arc::new(ChromaClient::new(Default::default()));
        Ok(Self {
            client,
            collection_name,
        })
    }

    pub async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()> {
        let collection = self.client.get_or_create_collection(&self.collection_name, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let doc_str = serde_json::to_string(article)
            .map_err(|e| nt_core::Error::Serialization(e))?;

        let metadata = serde_json::Map::from_iter(vec![
            ("url".to_string(), serde_json::Value::String(article.url.clone())),
            ("title".to_string(), serde_json::Value::String(article.title.clone())),
            ("source".to_string(), serde_json::Value::String(article.source.clone())),
            ("published_at".to_string(), serde_json::Value::String(article.published_at.to_rfc3339())),
            ("doc".to_string(), serde_json::Value::String(doc_str)),
        ]);

        let entries = CollectionEntries {
            ids: vec![&article.url],
            embeddings: Some(vec![embedding.to_vec()]),
            metadatas: Some(vec![metadata]),
            documents: None,
        };

        collection.add(entries, None)
            .map_err(|e| nt_core::Error::External(e))?;

        Ok(())
    }

    pub async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Article>> {
        let collection = self.client.get_or_create_collection(&self.collection_name, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let query_options = QueryOptions {
            query_embeddings: Some(vec![embedding.to_vec()]),
            query_texts: None,
            n_results: Some(limit),
            where_document: None,
            where_metadata: None,
            include: None,
        };

        let results = collection.query(query_options, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let mut articles = Vec::new();
        if let Some(metadatas) = results.metadatas {
            for metadata_vec in metadatas {
                if let Some(metadata_vec) = metadata_vec {
                    for metadata in metadata_vec {
                        if let Some(metadata) = metadata {
                            if let Some(doc_str) = metadata.get("doc").and_then(|v| v.as_str()) {
                                if let Ok(article) = serde_json::from_str::<Article>(doc_str) {
                                    articles.push(article);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(articles)
    }

    pub async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        let collection = self.client.get_or_create_collection(&self.collection_name, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let where_metadata = serde_json::Value::Object(serde_json::Map::from_iter(vec![
            ("source".to_string(), serde_json::Value::String(source.to_string())),
        ]));

        let query_options = QueryOptions {
            query_embeddings: Some(vec![vec![0.0; 384]]), // Dummy embedding for filtering
            query_texts: None,
            n_results: Some(100), // Adjust as needed
            where_document: None,
            where_metadata: Some(where_metadata),
            include: None,
        };

        let results = collection.query(query_options, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let mut articles = Vec::new();
        if let Some(metadatas) = results.metadatas {
            for metadata_vec in metadatas {
                if let Some(metadata_vec) = metadata_vec {
                    for metadata in metadata_vec {
                        if let Some(metadata) = metadata {
                            if let Some(doc_str) = metadata.get("doc").and_then(|v| v.as_str()) {
                                if let Ok(article) = serde_json::from_str::<Article>(doc_str) {
                                    articles.push(article);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(articles)
    }
}

pub struct ChromaStore {
    url: String,
    collection: String,
}

impl ChromaStore {
    pub fn new(url: String, collection: String) -> Self {
        Self {
            url,
            collection,
        }
    }

    pub async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()> {
        // TODO: Implement Chroma storage
        Ok(())
    }

    pub async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Article>> {
        // TODO: Implement Chroma similarity search
        Ok(Vec::new())
    }

    pub async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        // TODO: Implement Chroma source filtering
        Ok(Vec::new())
    }
}

pub struct ChromaStorage {
    store: Arc<RwLock<ChromaStore>>,
}

impl ChromaStorage {
    pub async fn new() -> Result<Self> {
        let config = ChromaConfig::new();
        let store = Arc::new(RwLock::new(ChromaStore::new(
            config.get_url(),
            config.get_collection(),
        )));
        Ok(Self { store })
    }
}

#[async_trait]
impl StorageBackend for ChromaStorage {
    fn get_error_message() -> &'static str {
        "ChromaDB should be running on http://localhost:8000"
    }

    async fn new() -> Result<Self> where Self: Sized {
        Self::new().await
    }
}

#[async_trait]
impl ArticleStorage for ChromaStorage {
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
    async fn test_chroma_storage() {
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

        let storage = ChromaStorage::new().await.unwrap();
        let embedding = vec![0.0; 384];
        storage.store_article(&article, &embedding).await.unwrap();
        let similar = storage.find_similar(&embedding, 1).await.unwrap();
        assert!(!similar.is_empty());
    }
} 