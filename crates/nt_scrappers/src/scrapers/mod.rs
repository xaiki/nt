use nt_core::{Article, Result, Error};
use scraper::{Html, Selector};
use url::Url;
use async_trait::async_trait;
use std::collections::HashMap;
use sha2::{Sha256, Digest};

pub mod argentina;

#[derive(Debug)]
pub enum ArticleStatus {
    New,
    Updated,
    Unchanged,
}

#[async_trait]
pub trait Scraper {
    /// Returns the name of the news source
    fn source(&self) -> &str;
    
    /// Returns true if this scraper can handle the given URL
    fn can_handle(&self, url: &str) -> bool;
    
    /// Scrapes an article from the given URL
    async fn scrape_article(&mut self, url: &str) -> Result<(Article, ArticleStatus)>;
    
    /// Returns a list of article URLs from the main page
    async fn get_article_urls(&self) -> Result<Vec<String>>;
}

/// Common utilities for scrapers
pub(crate) mod utils {
    use super::*;

    #[allow(dead_code)]
    pub fn parse_url(url: &str) -> Result<Url> {
        Url::parse(url).map_err(|e| Error::Scraping(format!("Failed to parse URL: {}", e)))
    }

    #[allow(dead_code)]
    pub fn extract_text(document: &Html, selector: &str) -> Result<String> {
        let selector = Selector::parse(selector)
            .map_err(|e| Error::Scraping(format!("Invalid selector: {}", e)))?;
        
        document
            .select(&selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .ok_or_else(|| Error::Scraping(format!("No element found for selector: {:?}", selector)))
    }

    #[allow(dead_code)]
    pub fn extract_texts(document: &Html, selector: &str) -> Result<Vec<String>> {
        let selector = Selector::parse(selector)
            .map_err(|e| Error::Scraping(format!("Invalid selector: {}", e)))?;
        
        Ok(document
            .select(&selector)
            .map(|el| el.text().collect::<String>())
            .collect())
    }

    #[allow(dead_code)]
    pub fn split_into_sections(content: &str) -> Vec<String> {
        content
            .split("\n\n")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

pub struct BaseScraper {
    article_cache: HashMap<String, String>,
}

impl BaseScraper {
    pub fn new() -> Self {
        Self {
            article_cache: HashMap::new(),
        }
    }

    pub fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn get_article_status(&self, url: &str, content: &str) -> ArticleStatus {
        let content_hash = Self::hash_content(content);
        match self.article_cache.get(url) {
            Some(old_hash) if old_hash == &content_hash => ArticleStatus::Unchanged,
            Some(_) => ArticleStatus::Updated,
            None => ArticleStatus::New,
        }
    }

    pub fn update_cache(&mut self, url: &str, content: &str) {
        let content_hash = Self::hash_content(content);
        self.article_cache.insert(url.to_string(), content_hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils;

    #[test]
    fn test_parse_url() {
        assert!(utils::parse_url("https://example.com").is_ok());
        assert!(utils::parse_url("invalid-url").is_err());
    }

    #[test]
    fn test_extract_text() {
        let html = r#"
            <div class="title">Test Title</div>
            <div class="content">Test Content</div>
        "#;
        let document = Html::parse_document(html);
        
        assert_eq!(
            utils::extract_text(&document, ".title").unwrap(),
            "Test Title"
        );
        assert!(utils::extract_text(&document, ".invalid").is_err());
    }

    #[test]
    fn test_extract_texts() {
        let html = r#"
            <div class="item">Item 1</div>
            <div class="item">Item 2</div>
        "#;
        let document = Html::parse_document(html);
        
        let texts = utils::extract_texts(&document, ".item").unwrap();
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0], "Item 1");
        assert_eq!(texts[1], "Item 2");
    }

    #[test]
    fn test_split_into_sections() {
        let content = "Section 1\n\nSection 2\n\n\nSection 3";
        let sections = utils::split_into_sections(content);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0], "Section 1");
        assert_eq!(sections[1], "Section 2");
        assert_eq!(sections[2], "Section 3");
    }
} 