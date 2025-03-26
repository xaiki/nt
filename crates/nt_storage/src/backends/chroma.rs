use async_trait::async_trait;
use nt_core::{Article, Result, storage::ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use chromadb::v1::{
    client::ChromaClient,
    collection::{CollectionEntries, QueryOptions},
};
use crate::StorageBackend;

#[async_trait::async_trait]
pub trait EmbeddingModel: Send + Sync {
    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>>;
}

pub struct DefaultEmbeddingModel;

#[async_trait::async_trait]
impl EmbeddingModel for DefaultEmbeddingModel {
    async fn generate_embeddings(&self, _text: &str) -> Result<Vec<f32>> {
        // For now, return a dummy embedding
        Ok(vec![0.0; 384])
    }
}

pub struct EmbeddingStore {
    client: Arc<ChromaClient>,
    collection_name: String,
    model: Arc<dyn EmbeddingModel>,
}

impl EmbeddingStore {
    pub fn new(collection_name: String, model: Arc<dyn EmbeddingModel>) -> Result<Self> {
        let client = Arc::new(ChromaClient::new(Default::default()));
        Ok(Self {
            client,
            collection_name,
            model,
        })
    }

    pub async fn store_article(&self, article: &Article) -> Result<()> {
        let collection = self.client.get_or_create_collection(&self.collection_name, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let doc_str = serde_json::to_string(article)
            .map_err(|e| nt_core::Error::Serialization(e))?;

        let embedding = self.model.generate_embeddings(&article.content).await?;

        let metadata = serde_json::Map::from_iter(vec![
            ("url".to_string(), serde_json::Value::String(article.url.clone())),
            ("title".to_string(), serde_json::Value::String(article.title.clone())),
            ("source".to_string(), serde_json::Value::String(article.source.clone())),
            ("published_at".to_string(), serde_json::Value::String(article.published_at.to_rfc3339())),
            ("doc".to_string(), serde_json::Value::String(doc_str)),
        ]);

        let entries = CollectionEntries {
            ids: vec![&article.url],
            embeddings: Some(vec![embedding]),
            metadatas: Some(vec![metadata]),
            documents: None,
        };

        collection.add(entries, None)
            .map_err(|e| nt_core::Error::External(e))?;

        Ok(())
    }

    pub async fn find_similar(&self, article: &Article, limit: usize) -> Result<Vec<Article>> {
        let collection = self.client.get_or_create_collection(&self.collection_name, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let embedding = self.model.generate_embeddings(&article.content).await?;

        let query_options = QueryOptions {
            query_embeddings: Some(vec![embedding]),
            query_texts: None,
            n_results: Some(limit),
            where_document: None,
            where_metadata: None,
            include: Some(vec!["metadatas"]),
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
            query_embeddings: Some(vec![vec![0.0; 384]]), // Dummy embedding
            query_texts: None,
            n_results: Some(100), // Adjust as needed
            where_document: None,
            where_metadata: Some(where_metadata),
            include: Some(vec!["metadatas"]),
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

pub struct ChromaDBStorage {
    store: Arc<RwLock<EmbeddingStore>>,
}

impl StorageBackend for ChromaDBStorage {
    fn get_error_message() -> &'static str {
        "ChromaDB should be running on http://localhost:8000"
    }

    async fn new() -> Result<Self> {
        let store = Arc::new(RwLock::new(EmbeddingStore::new(
            "articles".to_string(),
            Arc::new(DefaultEmbeddingModel),
        )?));
        Ok(Self { store })
    }
}

#[async_trait]
impl ArticleStorage for ChromaDBStorage {
    async fn store_article(&self, article: &Article) -> Result<()> {
        let store = self.store.read().await;
        store.store_article(article).await
    }

    async fn find_similar(&self, article: &Article, limit: usize) -> Result<Vec<Article>> {
        let store = self.store.read().await;
        store.find_similar(article, limit).await
    }

    async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        let store = self.store.read().await;
        store.get_by_source(source).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chroma_storage() {
        let storage = ChromaDBStorage::new().await.unwrap();
        let article = Article {
            url: "http://example.com".to_string(),
            title: "Test Article".to_string(),
            content: "Test content".to_string(),
            published_at: chrono::Utc::now(),
            source: "test".to_string(),
            sections: vec![],
            summary: None,
        };

        storage.store_article(&article).await.unwrap();
        let similar = storage.find_similar(&article, 1).await.unwrap();
        assert!(!similar.is_empty());
    }
} 