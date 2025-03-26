use std::sync::Arc;
use nt_core::{Article, Result};
use super::InferenceModel;

pub struct EmbeddingGenerator {
    model: Arc<dyn InferenceModel>,
}

impl EmbeddingGenerator {
    pub fn new(model: Arc<dyn InferenceModel>) -> Self {
        Self { model }
    }

    pub async fn generate_article_embedding(&self, article: &Article) -> Result<Vec<f32>> {
        self.model.generate_embeddings(&article.content).await
    }

    pub async fn generate_text_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.model.generate_embeddings(text).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::models::DeepSeekModel;

    #[tokio::test]
    async fn test_embedding_generation() {
        let model = Arc::new(DeepSeekModel::new(None).unwrap());
        let generator = EmbeddingGenerator::new(model);
        
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

        let embedding = generator.generate_article_embedding(&article).await.unwrap();
        assert!(!embedding.is_empty());

        let text_embedding = generator.generate_text_embedding("Test text").await.unwrap();
        assert!(!text_embedding.is_empty());
    }
} 