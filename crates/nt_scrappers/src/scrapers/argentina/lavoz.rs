use async_trait::async_trait;
use chrono::Utc;
use scraper::{Html, Selector};
use nt_core::{Result, Article, Scraper, SourceMetadata, ArticleSection};
use crate::scrapers::jsonld;
use super::REGION;

#[derive(Debug, Clone)]
pub struct LaVozScraper;

impl LaVozScraper {
    pub fn new() -> Self {
        Self
    }

    const BASE_URL: &'static str = "https://www.lavoz.com.ar";

    // Helper function to filter URLs
    fn filter_url(url: &str) -> bool {
        // Skip URLs that are clearly not articles
        if url.contains("/autor/")  // Skip author profile pages
          || url.contains("/mi-usuario/")
          || url.contains("/newsletter/")
          || url.contains("/404")  // Skip 404 pages
          || url.contains("/error") // Skip error pages
          || url.contains("?_ga=") // Skip tracking URLs
          || url.contains("voydeviaje") // Skip promotional travel content
          || url.contains("/club/") // Skip club section
          || url.contains("/beneficios/") // Skip benefits section
          || url.contains("/descuentos/") // Skip discounts section
          || url.contains("/avisos-") // Skip classified ads
          || url.split("/").count() < 3 // Skip URLs that are too short to be articles
          || url == Self::BASE_URL // Skip base URL
          || url == format!("{}/", Self::BASE_URL) { // Skip base URL with trailing slash
            return false;
        }

        // Get the last segment of the URL
        let last_segment = url.split('/').filter(|s| !s.is_empty()).last().unwrap_or("");
        
        // If the last segment has more than 2 hyphens, it's likely an article
        last_segment.chars().filter(|&c| c == '-').count() > 2
    }
}

#[async_trait]
impl Scraper for LaVozScraper {
    fn source_metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: "La Voz",
            emoji: "🧢",
            region: REGION,
        }
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("lavoz.com.ar")
    }

    fn cli_names(&self) -> Vec<&str> {
        vec!["lavoz"]
    }

    async fn scrape_article(&mut self, url: &str) -> Result<Article> {
        let response = reqwest::get(url).await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let title_selector = Selector::parse("h1").unwrap();
        let subtitle_selector = Selector::parse(".bajada").unwrap();
        let content_selector = Selector::parse(".body-nota p").unwrap();
        let date_selector = Selector::parse("time").unwrap();

        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let subtitle = document
            .select(&subtitle_selector)
            .next()
            .map(|el| el.text().collect::<String>());

        let mut authors = jsonld::extract_authors(&document);

        // If no authors found in JSON-LD, try HTML selectors
        if authors.is_empty() {
            // Try to get author from the text node after the date
            if let Some(date_element) = document.select(&date_selector).next() {
                if let Some(next_sibling) = date_element.next_sibling() {
                    if let Some(text) = next_sibling.value().as_text() {
                        let author_text = text.trim();
                        if !author_text.is_empty() && !author_text.contains("Compartir") {
                            authors.push(author_text.to_string());
                        }
                    }
                }
            }

            // If still no authors found, try searching for "Redacción LAVOZ"
            if authors.is_empty() {
                if let Ok(author_selector) = Selector::parse(".firma") {
                    for author in document.select(&author_selector) {
                        let author_text = author.text().collect::<String>().trim().to_string();
                        if !author_text.is_empty() && !author_text.contains("Compartir") {
                            authors.push(author_text);
                        }
                    }
                }
            }

            // If still no authors found, try searching through siblings
            if authors.is_empty() {
                if let Some(date_element) = document.select(&date_selector).next() {
                    if let Some(parent) = date_element.parent() {
                        for sibling in parent.next_siblings() {
                            if let Some(text) = sibling.value().as_text() {
                                let text = text.trim();
                                if !text.is_empty() && !text.contains("Compartir") {
                                    authors.push(text.to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut sections = Vec::new();
        
        // Add subtitle as first section if present
        if let Some(subtitle_text) = subtitle {
            if !subtitle_text.is_empty() {
                sections.push(ArticleSection {
                    content: subtitle_text,
                    summary: None,
                    embedding: None,
                });
            }
        }

        // Add article paragraphs
        for element in document.select(&content_selector) {
            let content = element.text().collect::<String>();
            if !content.is_empty() {
                sections.push(ArticleSection {
                    content,
                    summary: None,
                    embedding: None,
                });
            }
        }

        let content = sections
            .iter()
            .map(|s| s.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        let published_at = document
            .select(&date_selector)
            .next()
            .and_then(|el| el.value().attr("datetime"))
            .map(|date_str| chrono::DateTime::parse_from_rfc3339(date_str).ok())
            .flatten()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Ok(Article {
            url: url.to_string(),
            title,
            content,
            summary: None,
            published_at,
            source: self.source_metadata().name.to_string(),
            sections,
            authors,
        })
    }

    async fn get_article_urls(&self) -> Result<Vec<String>> {
        let mut urls = Vec::new();

        // Process latest news section
        {
            let response = reqwest::get(format!("{}/lo-ultimo/", Self::BASE_URL)).await?;
            let html = response.text().await?;
            let document = Html::parse_document(&html);
            
            if let Ok(article_selector) = Selector::parse("article.story-card") {
                for article in document.select(&article_selector) {
                    if let Some(h3) = article.select(&Selector::parse("h3").unwrap()).next() {
                        if let Some(link) = h3.select(&Selector::parse("a").unwrap()).next() {
                            if let Some(href) = link.value().attr("href") {
                                let url = if href.starts_with("http") {
                                    href.to_string()
                                } else {
                                    format!("{}{}", Self::BASE_URL, href)
                                };
                                if Self::filter_url(&url) {
                                    urls.push(url);
                                }
                            }
                        }
                    }
                }
            }
        }

        // If we didn't find enough articles, also check the main page
        if urls.len() < 10 {
            let response = reqwest::get(Self::BASE_URL).await?;
            let html = response.text().await?;
            let document = Html::parse_document(&html);
            
            if let Ok(article_selector) = Selector::parse("article.story-card") {
                for article in document.select(&article_selector) {
                    if let Some(h3) = article.select(&Selector::parse("h3").unwrap()).next() {
                        if let Some(link) = h3.select(&Selector::parse("a").unwrap()).next() {
                            if let Some(href) = link.value().attr("href") {
                                let url = if href.starts_with("http") {
                                    href.to_string()
                                } else {
                                    format!("{}{}", Self::BASE_URL, href)
                                };
                                if Self::filter_url(&url) {
                                    urls.push(url);
                                }
                            }
                        }
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
        let scraper = LaVozScraper::new();
        assert!(scraper.can_handle("https://www.lavoz.com.ar/article"));
        assert!(!scraper.can_handle("https://www.lanacion.com.ar/article"));
    }

    #[tokio::test]
    async fn test_scrape_article() {
        let mut scraper = LaVozScraper::new();
        let url = "https://www.lavoz.com.ar/politica/javier-milei-anuncio-superavit-fiscal-primer-trimestre-2024_0_MsAUOCyoYK.html";
        let result = scraper.scrape_article(url).await;
        assert!(result.is_ok());
        let article = result.unwrap();
        assert_eq!(article.url, url);
        assert!(!article.title.is_empty());
        assert!(!article.content.is_empty());
    }

    #[tokio::test]
    async fn test_get_article_urls() {
        let scraper = LaVozScraper::new();
        let urls = scraper.get_article_urls().await;
        assert!(urls.is_ok());
        let urls = urls.unwrap();
        assert!(!urls.is_empty());
        assert!(urls.iter().all(|url| url.contains("lavoz.com.ar")));
    }
} 