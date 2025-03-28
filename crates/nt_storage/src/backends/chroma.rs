use async_trait::async_trait;
use nt_core::{Article, Result, ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use chromadb::v1::{
    client::ChromaClient,
    collection::{CollectionEntries, QueryOptions},
};
use crate::{StorageBackend, BackendConfig, EmbeddingModel};
use std::env;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct ChromaConfig {
    pub config: BackendConfig,
}

impl ChromaConfig {
    pub fn new() -> Self {
        let host = env::var("CHROMA_HOST").unwrap_or_else(|_| "chroma".to_string());
        let port = env::var("CHROMA_PORT").unwrap_or_else(|_| "8000".to_string());
        let url = format!("http://{}:{}", host, port);
        Self {
            config: BackendConfig::new(
                url,
                "articles".to_string(),
                EmbeddingModel::default(),
                768,
            ),
        }
    }
}

impl Deref for ChromaConfig {
    type Target = BackendConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

pub struct EmbeddingStore {
    client: Arc<ChromaClient>,
    collection_name: String,
    vector_size: u64,
}

impl EmbeddingStore {
    pub fn new(collection_name: String, vector_size: u64) -> Result<Self> {
        let client = Arc::new(ChromaClient::new(Default::default()));
        Ok(Self {
            client,
            collection_name,
            vector_size,
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
            query_embeddings: Some(vec![vec![0.0; self.vector_size as usize]]), // Dummy embedding for filtering
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
    client: Arc<ChromaClient>,
    config: ChromaConfig,
}

impl ChromaStore {
    pub fn new(config: ChromaConfig) -> Result<Self> {
        let client = Arc::new(ChromaClient::new(Default::default()));
        Ok(Self {
            client,
            config,
        })
    }

    pub async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()> {
        let collection = self.client.get_or_create_collection(&self.config.collection, None)
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
        let collection = self.client.get_or_create_collection(&self.config.collection, None)
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
        let collection = self.client.get_or_create_collection(&self.config.collection, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let where_metadata = serde_json::Value::Object(serde_json::Map::from_iter(vec![
            ("source".to_string(), serde_json::Value::String(source.to_string())),
        ]));

        let query_options = QueryOptions {
            query_embeddings: Some(vec![vec![0.0; self.config.vector_size as usize]]), // Dummy embedding for filtering
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

pub struct ChromaStorage {
    store: Arc<RwLock<ChromaStore>>,
    config: ChromaConfig,
}

impl ChromaStorage {
    pub async fn new() -> Result<Self> {
        let config = ChromaConfig::new();
        let store = Arc::new(RwLock::new(ChromaStore::new(config.clone())?));
        Ok(Self { store, config })
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

    fn get_config(&mut self) -> Option<&mut BackendConfig> {
        Some(&mut self.config.config)
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

    async fn delete_article(&self, url: &str) -> Result<()> {
        let store = self.store.read().await;
        let collection = store.client.get_or_create_collection(&store.config.collection, None)
            .map_err(|e| nt_core::Error::External(e))?;
        
        collection.delete(Some(vec![url]), None, None)
            .map_err(|e| nt_core::Error::Database(format!("Failed to delete article: {}", e)))?;
        Ok(())
    }

    async fn get_article_embedding(&self, url: &str) -> Result<Vec<f32>> {
        let store = self.store.read().await;
        let collection = store.client.get_or_create_collection(&store.config.collection, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let query_options = QueryOptions {
            query_embeddings: Some(vec![vec![0.0; store.config.vector_size as usize]]), // Dummy embedding for filtering
            query_texts: None,
            n_results: Some(1),
            where_document: None,
            where_metadata: Some(serde_json::Value::Object(serde_json::Map::from_iter(vec![
                ("url".to_string(), serde_json::Value::String(url.to_string())),
            ]))),
            include: Some(vec!["embeddings"]),
        };

        let results = collection.query(query_options, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let embeddings = results.embeddings
            .ok_or_else(|| nt_core::Error::Database("No embeddings found in results".to_string()))?;

        if embeddings.is_empty() {
            return Err(nt_core::Error::Database("No embedding found for article".to_string()));
        }

        let embedding_vec = embeddings[0].as_ref().expect("No embedding vector found").first()
            .ok_or_else(|| nt_core::Error::Database("Empty embedding vector".to_string()))?
            .clone();

        Ok(embedding_vec)
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
            related_articles: Vec::new(),
        };

        let storage = ChromaStorage::new().await.unwrap();
        let vector_size = storage.config.vector_size;
        let embedding = vec![0.0; vector_size as usize];
        storage.store_article(&article, &embedding).await.unwrap();
        let similar = storage.find_similar(&embedding, 1).await.unwrap();
        assert!(!similar.is_empty());
    }
} 