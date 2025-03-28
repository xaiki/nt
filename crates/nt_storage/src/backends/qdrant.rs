use async_trait::async_trait;
use nt_core::{Article, Result, ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use qdrant_client::{
    prelude::*,
    qdrant::{
        vectors_config::Config, CreateCollectionBuilder, Distance, Filter, PointStruct, ScalarQuantizationBuilder, SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder, Condition,
        CreateCollection, DeleteCollection, GetCollectionInfoRequest, DeletePoints, PointsSelector, DeletePointsBuilder,
        VectorParams, VectorsConfig, PointId,
    },
    Payload, Qdrant,
};
use crate::{StorageBackend, BackendConfig, EmbeddingModel};
use std::env;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub config: BackendConfig,
}

impl QdrantConfig {
    pub fn new() -> Self {
        let host = env::var("QDRANT_HOST").unwrap_or_else(|_| "qdrant".to_string());
        let port = env::var("QDRANT_PORT").unwrap_or_else(|_| "6334".to_string());
        let url = format!("http://{}:{}", host, port);
        Self {
            config: BackendConfig::new(
                url,
                "articles".to_string(),
                EmbeddingModel::Qdrant,
                768,
            ),
        }
    }
}

impl Deref for QdrantConfig {
    type Target = BackendConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

pub struct QdrantStore {
    client: Arc<Qdrant>,
    config: QdrantConfig,
}

impl QdrantStore {
    pub async fn new(config: QdrantConfig) -> Result<Self> {
        let client = Qdrant::from_url(&config.url)
            .build()
            .map_err(|e| nt_core::Error::External(e.into()))?;
        let collection = config.collection.clone();

        // Check if collection exists and get its info
        let collections = client.list_collections()
            .await
            .map_err(|e| nt_core::Error::External(e.into()))?;
        
        let mut collection_exists = collections.collections.iter()
            .any(|c| c.name == collection);

        if collection_exists {
            // Get collection info to check vector size
            let collection_info = client.collection_info(GetCollectionInfoRequest {
                collection_name: collection.to_string(),
                ..Default::default()
            })
            .await
            .map_err(|e| nt_core::Error::External(e.into()))?;

            let vector_size = config.vector_size;
            
            if let Some(info) = collection_info.result {
                if let Some(config) = info.config {
                    if let Some(params) = config.params {
                        if let Some(vectors_config) = params.vectors_config {
                            if let Some(Config::Params(vector_params)) = vectors_config.config {
                                if vector_params.size as u64 != vector_size {
                                    // Delete collection if vector size doesn't match
                                    client.delete_collection(DeleteCollection {
                                        collection_name: collection.to_string(),
                                        ..Default::default()
                                    })
                                    .await
                                    .map_err(|e| nt_core::Error::External(e.into()))?;
                                    collection_exists = false;
                                }
                            }
                        }
                    }
                }
            }
        }

        if !collection_exists {
            // Create collection with correct vector size
            client
                .create_collection(
                    CreateCollectionBuilder::new(collection)
                        .vectors_config(VectorParamsBuilder::new(config.vector_size, Distance::Cosine))
                        .quantization_config(ScalarQuantizationBuilder::default()),
                )
                .await
                .map_err(|e| nt_core::Error::External(e.into()))?;
        }

        Ok(Self {
            client: Arc::new(client),
            config,
        })
    }

    pub async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()> {
        let doc_str = serde_json::to_string(article)
            .map_err(|e| nt_core::Error::Serialization(e))?;

        let mut payload = Payload::new();
        payload.insert("url", article.url.clone());
        payload.insert("title", article.title.clone());
        payload.insert("source", article.source.clone());
        payload.insert("published_at", article.published_at.to_rfc3339());
        payload.insert("doc", doc_str);

        let point = PointStruct::new(
            Uuid::new_v4().to_string(),
            embedding.to_vec(),
            payload
        );

        self.client.upsert_points(
            UpsertPointsBuilder::new(self.config.collection.clone(), vec![point])
        )
        .await
        .map_err(|e| nt_core::Error::External(e.into()))?;

        Ok(())
    }

    pub async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Article>> {
        let results = self.client.search_points(
            SearchPointsBuilder::new(
                self.config.collection.clone(),
                embedding.to_vec(),
                limit as u64
            )
            .with_payload(true)
        )
        .await
        .map_err(|e| nt_core::Error::External(e.into()))?;

        let mut articles = Vec::new();
        for point in results.result {
            if let Some(doc_str) = point.payload.get("doc").and_then(|v| v.as_str()) {
                if let Ok(article) = serde_json::from_str::<Article>(doc_str) {
                    articles.push(article);
                }
            }
        }

        Ok(articles)
    }

    pub async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        let results = self.client.search_points(
            SearchPointsBuilder::new(
                self.config.collection.clone(),
                vec![0.0; self.config.vector_size as usize],
                100
            )
            .with_payload(true)
            .filter(Filter::all([Condition::matches("source", source.to_string())]))
        )
        .await
        .map_err(|e| nt_core::Error::External(e.into()))?;

        let mut articles = Vec::new();
        for point in results.result {
            if let Some(doc_str) = point.payload.get("doc").and_then(|v| v.as_str()) {
                if let Ok(article) = serde_json::from_str::<Article>(doc_str) {
                    articles.push(article);
                }
            }
        }

        Ok(articles)
    }

    async fn create_collection(&self) -> Result<()> {
        let collection_name = self.config.collection.clone();
        let collection_info = self.client.collection_info(GetCollectionInfoRequest {
            collection_name: collection_name.clone(),
            ..Default::default()
        }).await;
        
        if collection_info.is_err() {
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(collection_name)
                        .vectors_config(VectorParamsBuilder::new(self.config.vector_size, Distance::Cosine))
                        .quantization_config(ScalarQuantizationBuilder::default()),
                )
                .await
                .map_err(|e| nt_core::Error::External(e.into()))?;
        }
        Ok(())
    }

    async fn delete_collection(&self) -> Result<()> {
        let collection_name = self.config.collection.clone();
        self.client.delete_collection(DeleteCollection {
            collection_name: collection_name.clone(),
            ..Default::default()
        })
        .await
        .map_err(|e| nt_core::Error::External(e.into()))?;
        Ok(())
    }

    pub async fn get_article_embedding(&self, url: &str) -> Result<Vec<f32>> {
        let results = self.client.search_points(
            SearchPointsBuilder::new(
                self.config.collection.clone(),
                vec![0.0; self.config.vector_size as usize],
                1
            )
            .with_payload(true)
            .filter(Filter::all([Condition::matches("url", url.to_string())]))
        )
        .await
        .map_err(|e| nt_core::Error::External(e.into()))?;

        let point = results.result.first()
            .ok_or_else(|| nt_core::Error::Database(format!("No embedding found for article: {}", url)))?;

        let doc_str = point.payload.get("doc")
            .ok_or_else(|| nt_core::Error::Database(format!("No document found for article: {}", url)))?
            .as_str()
            .ok_or_else(|| nt_core::Error::Database(format!("Invalid document format for article: {}", url)))?;

        let article: Article = serde_json::from_str(doc_str)
            .map_err(|e| nt_core::Error::Database(format!("Failed to deserialize article: {}", e)))?;

        Ok(article.sections.first()
            .ok_or_else(|| nt_core::Error::Database(format!("No sections found for article: {}", url)))?
            .embedding
            .as_ref()
            .ok_or_else(|| nt_core::Error::Database(format!("No embedding found for article: {}", url)))?
            .clone())
    }
}

pub struct QdrantStorage {
    store: Arc<RwLock<QdrantStore>>,
    config: QdrantConfig,
}

impl QdrantStorage {
    pub async fn new() -> Result<Self> {
        let config = QdrantConfig::new();
        let store = Arc::new(RwLock::new(QdrantStore::new(config.clone()).await?));
        Ok(Self { store, config })
    }
}

#[async_trait]
impl StorageBackend for QdrantStorage {
    fn get_error_message() -> &'static str {
        "Qdrant should be running on http://localhost:6333"
    }

    async fn new() -> Result<Self> where Self: Sized {
        let config = QdrantConfig::new();
        let store = Arc::new(RwLock::new(QdrantStore::new(config.clone()).await?));
        Ok(Self { store, config })
    }

    fn get_config(&mut self) -> Option<&mut BackendConfig> {
        Some(&mut self.config.config)
    }
}

#[async_trait]
impl ArticleStorage for QdrantStorage {
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
        
        // Try to delete but don't treat errors as fatal
        if let Err(e) = store.client.delete_points(
            DeletePoints {
                collection_name: store.config.collection.clone(),
                points: Some(PointsSelector::from(vec![PointId::from(url.to_string())])),
                ..Default::default()
            }
        ).await {
            tracing::warn!("Failed to delete article from Qdrant: {}", e);
            // Return Ok even if delete fails
        }
        Ok(())
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
    async fn test_qdrant_storage() {
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

        let storage = QdrantStorage::new().await.unwrap();
        let vector_size = storage.config.vector_size;
        let embedding = vec![0.0; vector_size as usize];
        storage.store_article(&article, &embedding).await.unwrap();
        let similar = storage.find_similar(&embedding, 1).await.unwrap();
        assert!(!similar.is_empty());
    }
} 