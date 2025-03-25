use async_trait::async_trait;
use crate::types::Article;
use crate::Result;

#[async_trait]
pub trait ArticleStorage: Send + Sync {
    /// Store an article
    async fn store_article(&self, article: &Article) -> Result<()>;

    /// Find similar articles based on embeddings
    async fn find_similar(&self, article: &Article, limit: usize) -> Result<Vec<Article>>;

    /// Get all articles from a specific source
    async fn get_by_source(&self, source: &str) -> Result<Vec<Article>>;
} 