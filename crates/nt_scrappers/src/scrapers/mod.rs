use async_trait::async_trait;
use nt_core::{Article, Result, Error, storage::ArticleStorage};
use scraper::{Html, Selector};
use url::Url;

pub mod argentina;
use argentina::clarin::ClarinScraper;
use argentina::lanacion::LaNacionScraper;
use argentina::lavoz::LaVozScraper;

#[derive(Debug)]
pub enum ArticleStatus {
    New,
    Updated,
    Unchanged,
}

#[async_trait]
pub trait Scraper: Send + Sync {
    /// Returns the name of the news source
    fn source(&self) -> &str;
    
    /// Returns true if this scraper can handle the given URL
    fn can_handle(&self, url: &str) -> bool;
    
    /// Scrapes an article from the given URL
    async fn scrape_article(&mut self, url: &str) -> Result<Article>;
    
    /// Returns a list of article URLs from the main page
    async fn get_article_urls(&self) -> Result<Vec<String>>;

    /// Returns a list of CLI shorthand names for this scraper
    fn cli_names(&self) -> Vec<&str> {
        vec![]
    }
}

/// Enum that holds all possible scraper types
#[derive(Clone)]
pub enum ScraperType {
    Clarin(ClarinScraper),
    LaNacion(LaNacionScraper),
    LaVoz(LaVozScraper),
}

impl ScraperType {
    pub fn source(&self) -> &str {
        match self {
            ScraperType::Clarin(s) => s.source(),
            ScraperType::LaNacion(s) => s.source(),
            ScraperType::LaVoz(s) => s.source(),
        }
    }

    pub fn can_handle(&self, url: &str) -> bool {
        match self {
            ScraperType::Clarin(s) => s.can_handle(url),
            ScraperType::LaNacion(s) => s.can_handle(url),
            ScraperType::LaVoz(s) => s.can_handle(url),
        }
    }

    pub async fn scrape_article(&mut self, url: &str) -> Result<Article> {
        match self {
            ScraperType::Clarin(s) => s.scrape_article(url).await,
            ScraperType::LaNacion(s) => s.scrape_article(url).await,
            ScraperType::LaVoz(s) => s.scrape_article(url).await,
        }
    }

    pub async fn get_article_urls(&self) -> Result<Vec<String>> {
        match self {
            ScraperType::Clarin(s) => s.get_article_urls().await,
            ScraperType::LaNacion(s) => s.get_article_urls().await,
            ScraperType::LaVoz(s) => s.get_article_urls().await,
        }
    }

    pub fn cli_names(&self) -> Vec<&str> {
        match self {
            ScraperType::Clarin(s) => s.cli_names(),
            ScraperType::LaNacion(s) => s.cli_names(),
            ScraperType::LaVoz(s) => s.cli_names(),
        }
    }
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

pub struct ScraperManager {
    storage: Box<dyn ArticleStorage>,
    scrapers: Vec<ScraperType>,
}

impl ScraperManager {
    pub fn new(storage: Box<dyn ArticleStorage>) -> Self {
        Self {
            storage,
            scrapers: Vec::new(),
        }
    }

    pub fn add_scraper(&mut self, scraper: ScraperType) {
        self.scrapers.push(scraper);
    }

    pub async fn scrape_url(&mut self, url: &str) -> Result<(Article, ArticleStatus)> {
        // Find a scraper that can handle this URL
        let scraper = self.scrapers
            .iter_mut()
            .find(|s| s.can_handle(url))
            .ok_or_else(|| Error::Scraping(format!("No scraper found for URL: {}", url)))?;

        // Scrape the article
        let article = scraper.scrape_article(url).await?;

        // Check if article exists in database
        let existing_articles = self.storage.get_by_source(scraper.source()).await?;
        let status = if let Some(existing) = existing_articles.iter().find(|a| a.url == url) {
            if existing.content == article.content {
                ArticleStatus::Unchanged
            } else {
                ArticleStatus::Updated
            }
        } else {
            ArticleStatus::New
        };

        // Store the article in the database
        self.storage.store_article(&article).await?;

        Ok((article, status))
    }

    pub async fn scrape_all(&mut self) -> Result<Vec<(Article, ArticleStatus)>> {
        let mut results = Vec::new();

        // Get all scrapers
        let scraper_sources: Vec<String> = self.scrapers.iter().map(|s| s.source().to_string()).collect();

        // For each scraper
        for source in scraper_sources {
            // Find the scraper again to get a mutable reference
            if let Some(scraper) = self.scrapers.iter_mut().find(|s| s.source() == source) {
                // Get all article URLs
                let urls = scraper.get_article_urls().await?;

                // Scrape each article
                for url in urls {
                    match self.scrape_url(&url).await {
                        Ok(result) => results.push(result),
                        Err(e) => eprintln!("Failed to scrape {}: {}", url, e),
                    }
                }
            }
        }

        Ok(results)
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