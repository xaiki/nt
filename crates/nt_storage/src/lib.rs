use async_trait::async_trait;
use nt_core::Result;
use std::fmt;
use std::fmt::Debug;
use std::any::Any;
use std::str::FromStr;

pub mod backends;

pub use backends::*;

#[async_trait]
pub trait StorageBackend: Send + Sync + Any {
    fn get_error_message() -> &'static str;
    async fn new() -> Result<Self> where Self: Sized;
    fn get_config(&mut self) -> Option<&mut BackendConfig>;
}

#[derive(Debug, Clone)]
pub enum EmbeddingModel {
    OpenAI,
    DeepSeek,
    Qdrant,
    SQLite,
}

impl Default for EmbeddingModel {
    fn default() -> Self {
        Self::DeepSeek
    }
}

pub trait UrlConfig {
    fn get_url(&self) -> String;
    fn with_url(&mut self, url: &str);
    fn get_host(&self) -> String;
    fn get_port(&self) -> u16;
}

impl UrlConfig for String {
    fn get_url(&self) -> String {
        self.clone()
    }

    fn with_url(&mut self, url: &str) {
        *self = url.to_string();
    }

    fn get_host(&self) -> String {
        self.split("://").nth(1)
            .unwrap_or("localhost")
            .split(":")
            .next()
            .unwrap_or("localhost")
            .to_string()
    }

    fn get_port(&self) -> u16 {
        self.split(":").last()
            .and_then(|p| p.parse().ok())
            .unwrap_or(80)
    }
}

#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub url: String,
    pub collection: String,
    pub embedding_model: EmbeddingModel,
    pub vector_size: u64,
}

impl BackendConfig {
    pub fn new(url: String, collection: String, embedding_model: EmbeddingModel, vector_size: u64) -> Self {
        Self {
            url,
            collection,
            embedding_model,
            vector_size,
        }
    }
}

impl UrlConfig for BackendConfig {
    fn get_url(&self) -> String {
        self.url.clone()
    }

    fn with_url(&mut self, url: &str) {
        self.url = url.to_string();
    }

    fn get_host(&self) -> String {
        self.url.get_host()
    }

    fn get_port(&self) -> u16 {
        self.url.get_port()
    }
}

pub trait ModelConfig: UrlConfig {
    fn get_model_name(&self) -> String;
    fn get_api_key(&self) -> Option<String>;
    fn get_vector_size(&self) -> u64;
}

pub mod prelude {
    pub use super::BackendConfig;
    pub use super::backends::*;
}
