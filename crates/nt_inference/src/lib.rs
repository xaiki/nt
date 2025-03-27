use nt_core::{Result, InferenceModel};
use nt_storage::BackendConfig;

pub mod models;
pub mod embeddings;
pub mod divergence;

#[derive(Debug)]
pub struct Config {
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub backend_config: Box<dyn BackendConfig>,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            api_key: self.api_key.clone(),
            model_name: self.model_name.clone(),
            backend_config: Box::new(nt_storage::backends::memory::MemoryConfig::new()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            model_name: None,
            backend_config: Box::new(nt_storage::backends::memory::MemoryConfig::new()),
        }
    }
}

pub mod prelude {
    pub use super::Config;
    pub use super::models::create_model;
    pub use nt_core::{Article, ArticleSection, Result, Error};
}

pub use models::create_model;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::models::deepseek::DeepSeekModel;
    use crate::divergence::DivergenceAnalyzer;

    #[tokio::test]
    async fn test_inference_pipeline() {
        let model = Arc::new(DeepSeekModel::new(None).unwrap());
        let _analyzer = DivergenceAnalyzer::new(model);
    }
}