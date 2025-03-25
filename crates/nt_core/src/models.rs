use async_trait::async_trait;
use crate::types::{Article, ArticleSection};
use crate::Result;

#[async_trait]
pub trait InferenceModel: Send + Sync {
    /// Summarize an entire article
    async fn summarize_article(&self, article: &Article) -> Result<String>;

    /// Summarize individual sections of an article
    async fn summarize_sections(&self, sections: &[ArticleSection]) -> Result<Vec<String>>;

    /// Generate embeddings for a piece of text
    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>>;
} 