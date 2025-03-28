use async_trait::async_trait;
use nt_core::{Result, Article, ArticleStorage};
use std::collections::HashMap;
use std::sync::Arc;
use std::str::FromStr;
use std::any::Any;

pub mod backends;
pub use backends::*;

pub use backends::memory::MemoryStorage as InMemoryStorage;
#[cfg(feature = "chroma")]
pub use backends::chroma::ChromaStorage;
#[cfg(feature = "qdrant")]
pub use backends::qdrant::QdrantStorage;
#[cfg(feature = "sqlite")]
pub use backends::sqlite::SQLiteStorage;

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

pub async fn get_available_storage_backends() -> HashMap<String, String> {
    let mut backends = HashMap::new();
    backends.insert("memory".to_string(), "In-memory storage".to_string());
    
    #[cfg(feature = "chroma")]
    backends.insert("chroma".to_string(), "ChromaDB vector database".to_string());
    
    #[cfg(feature = "qdrant")]
    backends.insert("qdrant".to_string(), "Qdrant vector database".to_string());
    
    #[cfg(feature = "sqlite")]
    backends.insert("sqlite".to_string(), "SQLite database".to_string());
    
    backends
}

pub async fn create_storage(backend: &str, url: Option<&str>) -> Result<Arc<dyn ArticleStorage>> {
    match backend {
        "memory" => {
            let mut storage = InMemoryStorage::new().await?;
            if let Some(url) = url {
                if let Some(config) = storage.get_config() {
                    config.with_url(url);
                }
            }
            Ok(Arc::new(storage))
        }
        #[cfg(feature = "chroma")]
        "chroma" => {
            let mut storage = ChromaStorage::new().await?;
            if let Some(url) = url {
                if let Some(config) = storage.get_config() {
                    config.with_url(url);
                }
            }
            Ok(Arc::new(storage))
        }
        #[cfg(feature = "qdrant")]
        "qdrant" => {
            let mut storage = QdrantStorage::new().await?;
            if let Some(url) = url {
                if let Some(config) = storage.get_config() {
                    config.with_url(url);
                }
            }
            Ok(Arc::new(storage))
        }
        #[cfg(feature = "sqlite")]
        "sqlite" => {
            let mut storage = SQLiteStorage::new().await?;
            if let Some(url) = url {
                if let Some(config) = storage.get_config() {
                    config.with_url(url);
                }
            }
            Ok(Arc::new(storage))
        }
        _ => {
            let backends = get_available_storage_backends().await;
            let available = backends.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
            Err(nt_core::Error::Storage(format!(
                "Unknown storage backend. Available backends: {}",
                available
            )))
        }
    }
}
