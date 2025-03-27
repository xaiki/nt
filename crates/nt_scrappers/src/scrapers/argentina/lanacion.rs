use async_trait::async_trait;
use chrono::Utc;
use scraper::{Html, Selector};
use nt_core::{Result, Article, Scraper, SourceMetadata, ArticleSection};
use crate::scrapers::jsonld;
use super::REGION;

#[derive(Debug, Clone)]
pub struct LaNacionScraper;

impl LaNacionScraper {
    pub fn new() -> Self {
        Self
    }

    const BASE_URL: &'static str = "https://www.lanacion.com.ar";
}

#[async_trait]
impl Scraper for LaNacionScraper {
    fn source_metadata(&self) -> SourceMetadata {
        SourceMetadata {
            name: "La Nación",
            emoji: "🧵",
            region: REGION,
        }
    }

    fn can_handle(&self, url: &str) -> bool {
        url.contains("lanacion.com.ar")
    }

    fn cli_names(&self) -> Vec<&str> {
        vec!["lanacion", "la-nacion"]
    }

    async fn scrape_article(&mut self, url: &str) -> Result<Article> {
        let response = reqwest::get(url).await?;
        
        // Check if we were redirected to the subscription page
        if response.url().to_string().contains("suscripciones.lanacion.com.ar") {
            return Err(nt_core::Error::Scraping("Article is subscription-only (redirected to suscripciones)".to_string()));
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Check for subscription-only content message
        if html.contains("Este contenido es sólo para suscriptores") {
            return Err(nt_core::Error::Scraping("Article is subscription-only".to_string()));
        }

        // Updated selectors for La Nacion's structure
        let title_selector = Selector::parse("h1").unwrap();
        let subtitle_selector = Selector::parse(".bajada, .copete").unwrap();
        let content_selector = Selector::parse(".cuerpo-nota p, article p").unwrap();
        let date_selector = Selector::parse("time").unwrap();
        let author_selector = Selector::parse(".com-autor, .autor, .com-txt-autor").unwrap();

        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let subtitle = document
            .select(&subtitle_selector)
            .next()
            .map(|el| el.text().collect::<String>());

        // Extract authors
        let mut authors = jsonld::extract_authors(&document);

        // If no authors found in JSON-LD, try HTML selectors
        if authors.is_empty() {
            for author_element in document.select(&author_selector) {
                let author_text = author_element.text().collect::<String>().trim().to_string();
                if !author_text.is_empty() {
                    authors.push(author_text);
                }
            }
        }

        // Try additional selectors if still no authors found
        if authors.is_empty() {
            if let Ok(selector) = Selector::parse("[data-author]") {
                for element in document.select(&selector) {
                    if let Some(author) = element.value().attr("data-author") {
                        authors.push(author.trim().to_string());
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
            published_at,
            source: self.source_metadata().name.to_string(),
            sections,
            summary: None,
            authors,
            related_articles: Vec::new(),
        })
    }

    async fn get_article_urls(&self) -> Result<Vec<String>> {
        let response = reqwest::get(Self::BASE_URL).await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);

        let mut urls = Vec::new();

        // Find all article links within ln-card articles
        if let Ok(link_selector) = Selector::parse("article.ln-card a.ln-link") {
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
        let scraper = LaNacionScraper::new();
        assert!(scraper.can_handle("https://www.lanacion.com.ar/article"));
        assert!(!scraper.can_handle("https://www.clarin.com/article"));
    }

    #[tokio::test]
    async fn test_scrape_article() {
        let mut scraper = LaNacionScraper::new();
        let url = "https://www.lanacion.com.ar/politica/javier-milei-anuncio-superavit-fiscal-primer-trimestre-2024_0_MsAUOCyoYK.html";
        let result = scraper.scrape_article(url).await;
        assert!(result.is_ok());
        let article = result.unwrap();
        assert_eq!(article.url, url);
        assert!(!article.title.is_empty());
        assert!(!article.content.is_empty());
    }

    #[tokio::test]
    async fn test_get_article_urls() {
        let scraper = LaNacionScraper::new();
        let urls = scraper.get_article_urls().await;
        assert!(urls.is_ok());
        let urls = urls.unwrap();
        assert!(!urls.is_empty());
        assert!(urls.iter().all(|url| url.contains("lanacion.com.ar")));
    }
} 