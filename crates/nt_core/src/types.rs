use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub url: String,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub published_at: DateTime<Utc>,
    pub source: String,
    pub sections: Vec<ArticleSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleSection {
    pub content: String,
    pub summary: Option<String>,
    pub embedding: Option<Vec<f32>>,
} 