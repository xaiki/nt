use nt_core::{Result, InferenceModel, UrlConfig};
use serde::{Deserialize, Serialize};

pub mod models;
pub mod embeddings;
pub mod divergence;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub model_url: Option<String>,
    pub backend_url: Option<String>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            model_url: None,
            backend_url: None,
        }
    }
}

impl UrlConfig for InferenceConfig {
    fn get_url(&self) -> String {
        self.model_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string())
    }

    fn with_url(&mut self, url: &str) {
        self.model_url = Some(url.to_string());
    }

    fn get_host(&self) -> String {
        let url = self.get_url();
        url.split("://").nth(1)
            .and_then(|s| s.split(':').next())
            .unwrap_or("localhost")
            .to_string()
    }

    fn get_port(&self) -> u16 {
        let url = self.get_url();
        url.split(':').nth(2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(11434)
    }
}

pub trait ModelConfig: Send + Sync + std::fmt::Debug {
    fn from_inference_config(config: &InferenceConfig) -> Self;
}

#[derive(Debug)]
pub struct Config {
    pub api_key: Option<String>,
    pub model_name: Option<String>,
    pub backend_config: nt_storage::BackendConfig,
    pub inference_config: InferenceConfig,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            api_key: self.api_key.clone(),
            model_name: self.model_name.clone(),
            backend_config: self.backend_config.clone(),
            inference_config: self.inference_config.clone(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            model_name: None,
            backend_config: nt_storage::backends::memory::MemoryConfig::new().config,
            inference_config: InferenceConfig::default(),
        }
    }
}

pub mod prelude {
    pub use super::{Config, ModelConfig, InferenceConfig};
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