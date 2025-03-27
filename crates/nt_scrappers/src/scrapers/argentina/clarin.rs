use async_trait::async_trait;
use chrono::Utc;
use scraper::{Html, Selector};
use nt_core::{Result};
use nt_core::types::Article;
use crate::scrapers::{Scraper, SourceMetadata};
use serde_json;
use super::REGION;

#[derive(Debug, Clone)]
pub struct ClarinScraper;

impl ClarinScraper {
    pub fn new() -> Self {
        Self
    }

    const BASE_URL: &'static str = "https://www.clarin.com";
}

#[async_trait]
impl Scraper for ClarinScraper {
    fn source_metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: "Clarín",
            emoji: "🔇",
            region: REGION,
        }
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("clarin.com")
    }

    fn cli_names(&self) -> Vec<&str> {
        vec!["clarin"]
    }

    async fn scrape_article(&mut self, url: &str) -> Result<Article> {
        let response = reqwest::get(url).await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let title = document
            .select(&Selector::parse("h1").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let content = document
            .select(&Selector::parse("div[data-testid='body-text']").unwrap())
            .map(|el| el.text().collect::<String>().trim().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        let authors = if url.contains("elle.clarin.com") {
            // For Elle articles, get author from meta tag
            document
                .select(&Selector::parse("meta[name='author']").unwrap())
                .next()
                .and_then(|el| el.value().attr("content"))
                .map(|author| vec![author.to_string()])
                .unwrap_or_default()
        } else {
            // For regular Clarín articles, try multiple author sources
            let mut authors = Vec::new();
            
            // Try getting authors from script tag with article data
            if let Ok(script_selector) = Selector::parse("script[type='application/ld+json']") {
                for script in document.select(&script_selector) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(script.text().collect::<String>().trim()) {
                        // First try the discover array which contains article metadata
                        if let Some(discover) = json.get("discover") {
                            if let Some(discover_array) = discover.as_array() {
                                for item in discover_array {
                                    if let Some(article_authors) = item.get("authors") {
                                        if let Some(authors_array) = article_authors.as_array() {
                                            for author in authors_array {
                                                if let Some(name) = author.get("name") {
                                                    if let Some(name_str) = name.as_str() {
                                                        authors.push(name_str.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        // If no authors found in discover array, try the old format
                        if authors.is_empty() {
                            if let Some(author) = json.get("author") {
                                // Try array format first
                                if let Some(authors_array) = author.as_array() {
                                    for author_obj in authors_array {
                                        if let Some(name) = author_obj.get("name") {
                                            if let Some(name_str) = name.as_str() {
                                                authors.push(name_str.to_string());
                                            }
                                        }
                                    }
                                } else {
                                    // Try single author object
                                    if let Some(name) = author.get("name") {
                                        if let Some(name_str) = name.as_str() {
                                            authors.push(name_str.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // If no authors found in JSON, try .firma class
            if authors.is_empty() {
                authors = document
                    .select(&Selector::parse(".firma").unwrap())
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .filter(|text| !text.is_empty())
                    .collect();
            }
            
            authors
        };

        Ok(Article {
            title,
            content,
            url: url.to_string(),
            authors,
            published_at: Utc::now(),
            source: self.source_metadata().name.to_string(),
            sections: vec![],
            summary: None,
        })
    }

    async fn get_article_urls(&self) -> Result<Vec<String>> {
        let response = reqwest::get(Self::BASE_URL).await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let mut urls = Vec::new();

        // Find all article links
        if let Ok(link_selector) = Selector::parse("article a") {
            for link in document.select(&link_selector) {
                if let Some(href) = link.value().attr("href") {
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("{}{}", Self::BASE_URL, href)
                    };

                    // Skip URLs that are clearly not articles
                    if url.contains("/club/") 
                       || url.contains("/ayuda/")
                       || url.contains("/colecciones/")
                       || url.contains("/edicionimpresa/")
                       || url.contains("/foodit/")
                       || url.contains("/lncampo/")
                       || url.contains("/lnmas/")
                       || url.contains("/masmusica/")
                       || url.contains("/myaccount/")
                       || url.contains("/newsletter/")
                       || url.contains("/pdf/")
                       || url.contains("/servicios/")
                       || url.contains("/canchallena/")
                       || url.contains("/mi-usuario/")
                       || url.contains("?_ga=") // Skip tracking URLs
                       || url.contains("/trucos/")
                       || url.contains("/masterclass/")
                       || url.contains("/remates")
                       || url.contains("/avisos-")
                       || url.contains("/beneficios")
                       || url.contains("/descuentos") {
                        continue;
                    }

                    // Only include URLs that look like article URLs
                    // Check for at least one slash and no double slashes in the path part
                    if url.contains("/") && !url.split_once("://").map_or(false, |(_, path)| path.contains("//")) {
                        urls.push(url);
                    }
                }
            }
        }

        // Remove duplicates while preserving order
        urls.sort();
        urls.dedup();

        Ok(urls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_can_handle() {
        let scraper = ClarinScraper::new();
        assert!(scraper.can_handle("https://www.clarin.com/article"));
        assert!(!scraper.can_handle("https://www.lanacion.com.ar/article"));
    }

    #[tokio::test]
    async fn test_scrape_article() {
        let mut scraper = ClarinScraper::new();
        let url = "https://www.clarin.com/politica/javier-milei-anuncio-superavit-fiscal-primer-trimestre-2024_0_MsAUOCyoYK.html";
        let result = scraper.scrape_article(url).await;
        assert!(result.is_ok());
        let article = result.unwrap();
        assert_eq!(article.url, url);
        assert!(!article.title.is_empty());
        assert!(!article.content.is_empty());
    }

    #[tokio::test]
    async fn test_get_article_urls() {
        let scraper = ClarinScraper::new();
        let urls = scraper.get_article_urls().await;
        assert!(urls.is_ok());
        let urls = urls.unwrap();
        assert!(!urls.is_empty());
        assert!(urls.iter().all(|url| url.contains("clarin.com")));
    }
} 