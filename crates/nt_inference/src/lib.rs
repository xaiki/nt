use async_trait::async_trait;
use nt_core::{Article, ArticleSection, Result};

pub mod models;
pub mod embeddings;
pub mod divergence;

#[async_trait]
pub trait InferenceModel: Send + Sync {
    /// Returns the name of the model
    fn name(&self) -> &str;
    
    /// Generates a summary for the given article
    async fn summarize_article(&self, article: &Article) -> Result<String>;
    
    /// Generates summaries for each section of the article
    async fn summarize_sections(&self, sections: &[ArticleSection]) -> Result<Vec<String>>;
    
    /// Generates embeddings for the given text
    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>>;
}

pub mod prelude {
    pub use super::InferenceModel;
    pub use nt_core::{Article, ArticleSection, Result, Error};
}

/// Configuration for the inference models
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub model_name: String,
    pub model_path: Option<String>,
    pub api_key: Option<String>,
} 

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::models::DeepSeekModel;
    use crate::divergence::DivergenceAnalyzer;

    #[tokio::test]
    async fn test_inference_pipeline() {
        let model = Arc::new(DeepSeekModel::new(None).unwrap());
        let _analyzer = DivergenceAnalyzer::new(model);
    }
}