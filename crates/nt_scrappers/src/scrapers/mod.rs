use async_trait::async_trait;
use nt_core::{Article, Result, Error, SourceMetadata, Scraper};
use scraper::{Html, Selector};
use url::Url;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub mod argentina;

type BoxedScraper = Box<dyn Scraper + Send + Sync>;

pub type ScraperFactory = Box<dyn Fn() -> Box<dyn Scraper + Send + Sync> + Send + Sync>;

#[derive(Debug, Clone)]
pub enum ScraperType {
    Clarin(argentina::ClarinScraper),
    LaNacion(argentina::LaNacionScraper),
    LaVoz(argentina::LaVozScraper),
}

#[async_trait]
impl Scraper for ScraperType {
    fn source_metadata(&self) -> SourceMetadata {
        match self {
            ScraperType::Clarin(s) => s.source_metadata(),
            ScraperType::LaNacion(s) => s.source_metadata(),
            ScraperType::LaVoz(s) => s.source_metadata(),
        }
    }

    fn can_handle(&self, url: &str) -> bool {
        match self {
            ScraperType::Clarin(s) => s.can_handle(url),
            ScraperType::LaNacion(s) => s.can_handle(url),
            ScraperType::LaVoz(s) => s.can_handle(url),
        }
    }

    fn cli_names(&self) -> Vec<&str> {
        match self {
            ScraperType::Clarin(s) => s.cli_names(),
            ScraperType::LaNacion(s) => s.cli_names(),
            ScraperType::LaVoz(s) => s.cli_names(),
        }
    }

    async fn scrape_article(&mut self, url: &str) -> Result<Article> {
        match self {
            ScraperType::Clarin(s) => s.scrape_article(url).await,
            ScraperType::LaNacion(s) => s.scrape_article(url).await,
            ScraperType::LaVoz(s) => s.scrape_article(url).await,
        }
    }

    async fn get_article_urls(&self) -> Result<Vec<String>> {
        match self {
            ScraperType::Clarin(s) => s.get_article_urls().await,
            ScraperType::LaNacion(s) => s.get_article_urls().await,
            ScraperType::LaVoz(s) => s.get_article_urls().await,
        }
    }
}

impl ScraperType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "clarin" => Some(ScraperType::Clarin(argentina::ClarinScraper::new())),
            "lanacion" => Some(ScraperType::LaNacion(argentina::LaNacionScraper::new())),
            "lavoz" => Some(ScraperType::LaVoz(argentina::LaVozScraper::new())),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ScraperType::Clarin(_) => "clarin".to_string(),
            ScraperType::LaNacion(_) => "lanacion".to_string(),
            ScraperType::LaVoz(_) => "lavoz".to_string(),
        }
    }
}

pub fn get_scrapers() -> Vec<Arc<Mutex<BoxedScraper>>> {
    vec![
        Arc::new(Mutex::new(Box::new(ScraperType::Clarin(argentina::ClarinScraper::new())))),
        Arc::new(Mutex::new(Box::new(ScraperType::LaNacion(argentina::LaNacionScraper::new())))),
        Arc::new(Mutex::new(Box::new(ScraperType::LaVoz(argentina::LaVozScraper::new())))),
    ]
}

pub fn get_scraper_factories() -> Vec<ScraperFactory> {
    vec![
        Box::new(|| Box::new(crate::scrapers::argentina::ClarinScraper::new())),
        Box::new(|| Box::new(crate::scrapers::argentina::LaNacionScraper::new())),
        Box::new(|| Box::new(crate::scrapers::argentina::LaVozScraper::new())),
    ]
}

pub mod jsonld;

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