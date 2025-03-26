use async_trait::async_trait;
use nt_core::{Article, Result, storage::ArticleStorage};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub mod backends;

pub trait StorageBackend: ArticleStorage {
    fn get_error_message() -> &'static str;
    async fn new() -> Result<Self> where Self: Sized;
}

pub use backends::InMemoryStorage;
#[cfg(feature = "qdrant")]
pub use backends::QdrantStorage;
#[cfg(feature = "chroma")]
pub use backends::ChromaDBStorage;
