use async_trait::async_trait;
use nt_core::{Article, Result, ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollection, Distance, PointStruct, SearchPoints,
        VectorParams, VectorsConfig, vectors_config::Config, WithPayloadSelector,
        FieldCondition, Filter, Match, Vectors,
        r#match::MatchValue, UpsertPoints,
    },
};
use std::collections::HashMap;
use crate::{StorageBackend, BackendConfig, EmbeddingModel};
use std::env;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub collection: String,
}

impl QdrantConfig {
    pub fn new() -> Self {
        let host = env::var("QDRANT_HOST").unwrap_or_else(|_| "qdrant".to_string());
        let url = format!("http://{}:6334", host);
        Self {
            url,
            collection: "articles".to_string(),
        }
    }
}

impl BackendConfig for QdrantConfig {
    fn get_url(&self) -> String {
        self.url.clone()
    }

    fn get_collection(&self) -> String {
        self.collection.clone()
    }

    fn get_embedding_model(&self) -> EmbeddingModel {
        EmbeddingModel::Qdrant
    }
}

pub struct QdrantStore {
    client: Arc<Qdrant>,
    collection_name: String,
}

impl QdrantStore {
    pub async fn new(collection_name: String) -> Result<Self> {
        let host = env::var("QDRANT_HOST").unwrap_or_else(|_| "qdrant".to_string());
        let client = Qdrant::from_url(&format!("http://{}:6334", host))
            .build()
            .map_err(|e| nt_core::Error::External(e.into()))?;
        let client = Arc::new(client);
        
        let collections = client.list_collections()
            .await
            .map_err(|e| nt_core::Error::External(e.into()))?;

        if !collections.collections.iter().any(|c| c.name == collection_name) {
            let vector_config = VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: 384,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            };

            client.create_collection(CreateCollection {
                collection_name: collection_name.clone(),
                vectors_config: Some(vector_config),
                ..Default::default()
            })
            .await
            .map_err(|e| nt_core::Error::External(e.into()))?;
        }

        Ok(Self {
            client,
            collection_name,
        })
    }

    pub async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()> {
        let doc_str = serde_json::to_string(article)
            .map_err(|e| nt_core::Error::Serialization(e))?;

        let mut payload = HashMap::new();
        payload.insert("url".to_string(), article.url.clone().into());
        payload.insert("title".to_string(), article.title.clone().into());
        payload.insert("source".to_string(), article.source.clone().into());
        payload.insert("published_at".to_string(), article.published_at.to_rfc3339().into());
        payload.insert("doc".to_string(), doc_str.into());

        let point = PointStruct {
            id: Some(Uuid::new_v4().to_string().into()),
            vectors: Some(Vectors::from(embedding.to_vec())),
            payload: payload,
        };

        let points = vec![point];
        self.client.upsert_points(UpsertPoints {
            collection_name: self.collection_name.clone(),
            points,
            ..Default::default()
        })
            .await
            .map_err(|e| nt_core::Error::External(e.into()))?;

        Ok(())
    }

    pub async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Article>> {
        let search_request = SearchPoints {
            collection_name: self.collection_name.clone(),
            vector: embedding.to_vec(),
            limit: limit as u64,
            with_payload: Some(WithPayloadSelector::from(true)),
            ..Default::default()
        };

        let results = self.client.search_points(search_request)
            .await
            .map_err(|e| nt_core::Error::External(e.into()))?;

        let mut articles = Vec::new();
        for point in results.result {
            let payload = point.payload;
            if let Some(doc_str) = payload.get("doc").and_then(|v| v.as_str()) {
                if let Ok(article) = serde_json::from_str::<Article>(doc_str) {
                    articles.push(article);
                }
            }
        }

        Ok(articles)
    }

    pub async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        let search_request = SearchPoints {
            collection_name: self.collection_name.clone(),
            vector: vec![0.0; 384],
            limit: 100,
            with_payload: Some(WithPayloadSelector::from(true)),
            filter: Some(Filter {
                must: vec![FieldCondition {
                    key: "source".to_string(),
                    r#match: Some(Match {
                        match_value: Some(MatchValue::Text(source.to_string())),
                    }),
                    ..Default::default()
                }.into()],
                ..Default::default()
            }),
            ..Default::default()
        };

        let results = self.client.search_points(search_request)
            .await
            .map_err(|e| nt_core::Error::External(e.into()))?;

        let mut articles = Vec::new();
        for point in results.result {
            let payload = point.payload;
            if let Some(doc_str) = payload.get("doc").and_then(|v| v.as_str()) {
                if let Ok(article) = serde_json::from_str::<Article>(doc_str) {
                    articles.push(article);
                }
            }
        }

        Ok(articles)
    }
}

pub struct QdrantStorage {
    store: Arc<RwLock<QdrantStore>>,
}

impl QdrantStorage {
    pub async fn new() -> Result<Self> {
        let store = Arc::new(RwLock::new(QdrantStore::new("articles".to_string()).await?));
        Ok(Self { store })
    }
}

#[async_trait]
impl StorageBackend for QdrantStorage {
    fn get_error_message() -> &'static str {
        "Qdrant should be running on http://localhost:6333"
    }

    async fn new() -> Result<Self> where Self: Sized {
        let store = Arc::new(RwLock::new(QdrantStore::new("articles".to_string()).await?));
        Ok(Self { store })
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
        };

        let storage = QdrantStorage::new().await.unwrap();
        let embedding = vec![0.0; 384];
        storage.store_article(&article, &embedding).await.unwrap();
        let similar = storage.find_similar(&embedding, 1).await.unwrap();
        assert!(!similar.is_empty());
    }
} 