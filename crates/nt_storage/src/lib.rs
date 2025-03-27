use async_trait::async_trait;
use nt_core::Result;
use std::fmt;
use std::fmt::Debug;

pub mod backends;

pub use backends::*;

#[async_trait]
pub trait StorageBackend: Send + Sync {
    fn get_error_message() -> &'static str;
    async fn new() -> Result<Self> where Self: Sized;
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

pub trait BackendConfig: fmt::Debug {
    fn get_url(&self) -> String;
    fn get_collection(&self) -> String;
    fn get_embedding_model(&self) -> EmbeddingModel;
}

pub mod prelude {
    pub use super::BackendConfig;
    pub use super::backends::*;
}
