use async_trait::async_trait;
use chrono::Utc;
use scraper::{Html, Selector};
use nt_core::{Article, ArticleSection, Result};
use crate::scrapers::{Scraper, BaseScraper, ArticleStatus};

pub struct ClarinScraper {
    base: BaseScraper,
}

impl ClarinScraper {
    pub fn new() -> Self {
        Self {
            base: BaseScraper::new(),
        }
    }

    const BASE_URL: &'static str = "https://www.clarin.com";
}

#[async_trait]
impl Scraper for ClarinScraper {
    fn source(&self) -> &str {
        "Clarín"
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("clarin.com")
    }

    async fn scrape_article(&mut self, url: &str) -> Result<(Article, ArticleStatus)> {
        let response = reqwest::get(url).await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let title_selector = Selector::parse("h1").unwrap();
        let content_selector = Selector::parse("article p").unwrap();

        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let mut sections = Vec::new();
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

        let article = Article {
            url: url.to_string(),
            title,
            content: content.clone(),
            summary: None,
            published_at: Utc::now(),
            source: self.source().to_string(),
            sections,
        };

        let status = self.base.get_article_status(url, &content);
        self.base.update_cache(url, &content);
        Ok((article, status))
    }

    async fn get_article_urls(&self) -> Result<Vec<String>> {
        let response = reqwest::get(Self::BASE_URL).await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Try multiple selectors that might contain article links
        let selectors = [
            "article a",
            ".article-title a",
            ".article-link",
            ".article a"
        ];

        let mut urls = Vec::new();
        for selector in selectors.iter() {
            if let Ok(link_selector) = Selector::parse(selector) {
                let found_urls = document
                    .select(&link_selector)
                    .filter_map(|el| el.value().attr("href"))
                    .map(|href| {
                        if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("{}{}", Self::BASE_URL, href)
                        }
                    })
                    .filter(|url| {
                        url.contains("clarin.com") && 
                        !url.contains("/tag/") &&
                        !url.contains("colecciones.clarin.com") &&
                        !url.contains("comercial.clarin.com") &&
                        !url.contains("receptoriaonline.clarin.com") &&
                        !url.contains("clasificados") &&
                        !url.contains("beneficios") &&
                        !url.contains("trabajo") &&
                        url.contains("/policiales/") || 
                        url.contains("/politica/") || 
                        url.contains("/economia/") || 
                        url.contains("/sociedad/") ||
                        url.contains("/deportes/") ||
                        url.contains("/mundo/")
                    })
                    .collect::<Vec<_>>();
                urls.extend(found_urls);
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
        let (article, status) = result.unwrap();
        assert!(matches!(status, ArticleStatus::New));
        assert_eq!(article.url, url);
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