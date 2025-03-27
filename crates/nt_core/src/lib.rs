use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use thiserror::Error;
use std::any::Any;

pub trait UrlConfig {
    fn get_url(&self) -> String;
    fn with_url(&mut self, url: &str);
    fn get_host(&self) -> String;
    fn get_port(&self) -> u16;
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Scraping error: {0}")]
    Scraping(String),
    
    #[error("Inference error: {0}")]
    Inference(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("External error: {0}")]
    External(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub url: String,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub published_at: DateTime<Utc>,
    pub source: String,
    pub sections: Vec<ArticleSection>,
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleSection {
    pub content: String,
    pub summary: Option<String>,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArticleStatus {
    New,
    Updated,
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct RegionMetadata {
    pub name: &'static str,
    pub emoji: &'static str,
}

#[derive(Debug, Clone)]
pub struct SourceMetadata {
    pub name: &'static str,
    pub emoji: &'static str,
    pub region: RegionMetadata,
}

#[async_trait]
pub trait Scraper: Send + Sync {
    /// Returns metadata about the news source
    fn source_metadata(&self) -> SourceMetadata;
    
    /// Returns true if this scraper can handle the given URL
    fn can_handle(&self, url: &str) -> bool;
    
    /// Scrapes an article from the given URL
    async fn scrape_article(&mut self, url: &str) -> Result<Article>;
    
    /// Returns a list of article URLs from the main page
    async fn get_article_urls(&self) -> Result<Vec<String>>;

    /// Returns a list of CLI shorthand names for this scraper
    fn cli_names(&self) -> Vec<&str>;
}

#[async_trait]
pub trait InferenceModel: Send + Sync {
    /// Returns the name of the model
    fn name(&self) -> &str;

    /// Summarize an entire article
    async fn summarize_article(&self, article: &Article) -> Result<String>;

    /// Summarize individual sections of an article
    async fn summarize_sections(&self, sections: &[ArticleSection]) -> Result<Vec<String>>;

    /// Generate embeddings for a piece of text
    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait]
pub trait ArticleStorage: Send + Sync + Any {
    /// Store an article with its embedding
    async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()>;

    /// Find similar articles based on embedding
    async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Article>>;

    /// Get all articles from a specific source
    async fn get_by_source(&self, source: &str) -> Result<Vec<Article>>;
} 