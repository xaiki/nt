use thiserror::Error;

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