use std::sync::Arc;
use nt_core::{Article, Result};
use super::InferenceModel;
use chromadb::v1::{
    client::{ChromaClient, ChromaClientOptions},
    collection::{CollectionEntries, QueryOptions},
};

pub struct EmbeddingStore {
    client: Arc<ChromaClient>,
    model: Arc<dyn InferenceModel>,
    collection_name: String,
}

impl EmbeddingStore {
    pub fn new(model: Arc<dyn InferenceModel>, collection_name: String) -> Result<Self> {
        let client = Arc::new(ChromaClient::new(ChromaClientOptions::default()));
        Ok(Self {
            client,
            model,
            collection_name,
        })
    }

    pub async fn store_article(&self, article: &Article) -> Result<()> {
        let collection = self.client.get_or_create_collection(&self.collection_name, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let embedding = self.model.generate_embeddings(&article.content).await?;
        let doc_str = serde_json::to_string(article)
            .map_err(|e| nt_core::Error::Serialization(e))?;
        
        let entries = CollectionEntries {
            ids: vec![&article.url],
            embeddings: Some(vec![embedding]),
            documents: Some(vec![&doc_str]),
            metadatas: None,
        };

        collection.add(entries, None)
            .map_err(|e| nt_core::Error::External(e))?;

        Ok(())
    }

    pub async fn find_similar(&self, article: &Article, limit: usize) -> Result<Vec<Article>> {
        let collection = self.client.get_or_create_collection(&self.collection_name, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let embedding = self.model.generate_embeddings(&article.content).await?;
        
        let options = QueryOptions {
            query_embeddings: Some(vec![embedding]),
            query_texts: None,
            n_results: Some(limit),
            where_metadata: None,
            where_document: None,
            include: None,
        };

        let results = collection.query(options, None)
            .map_err(|e| nt_core::Error::External(e))?;

        let mut articles = Vec::new();
        
        if let Some(docs) = results.documents {
            for doc_vec in docs {
                if let Some(inner_vec) = doc_vec {
                    for doc in inner_vec {
                        if let Some(doc_str) = doc {
                            if let Ok(article) = serde_json::from_str::<Article>(&doc_str) {
                                articles.push(article);
                            }
                        }
                    }
                }
            }
        }

        Ok(articles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::models::DeepSeekModel;

    #[tokio::test]
    async fn test_store_and_find_similar() {
        let model = Arc::new(DeepSeekModel::new(None).unwrap());
        let store = EmbeddingStore::new(model, "test_collection".to_string()).unwrap();
        
        let article = Article {
            url: "http://example.com".to_string(),
            title: "Test Article".to_string(),
            content: "Test content".to_string(),
            published_at: chrono::Utc::now(),
            source: "test".to_string(),
            sections: vec![],
            summary: None,
        };

        store.store_article(&article).await.unwrap();
        let similar = store.find_similar(&article, 1).await.unwrap();
        assert_eq!(similar.len(), 1);
        assert_eq!(similar[0].url, article.url);
    }
} 