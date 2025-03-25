pub mod models;
pub mod error;
pub mod storage;
pub mod types;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Article {
    pub url: String,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub sections: Vec<ArticleSection>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArticleSection {
    pub content: String,
    pub summary: Option<String>,
    pub embedding: Option<Vec<f32>>,
} 