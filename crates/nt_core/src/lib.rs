pub mod models;
pub mod error;
pub mod storage;
pub mod types;

pub use error::Error;
pub use types::{Article, ArticleSection};
pub type Result<T> = std::result::Result<T, Error>; 