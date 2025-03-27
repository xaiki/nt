use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use nt_core::{Article, Result, Error, ArticleStorage, InferenceModel, ArticleStatus, Scraper};
use crate::scrapers::ScraperType;

pub struct ScraperManager {
    storage: Arc<dyn ArticleStorage>,
    inference: Arc<dyn InferenceModel>,
    scrapers: Vec<Arc<Mutex<ScraperType>>>,
}

impl ScraperManager {
    pub async fn new(storage: Arc<dyn ArticleStorage>, inference: Arc<dyn InferenceModel>) -> Result<Self> {
        Ok(Self {
            storage,
            inference,
            scrapers: Vec::new(),
        })
    }

    pub fn add_scraper(&mut self, scraper: ScraperType) {
        self.scrapers.push(Arc::new(Mutex::new(scraper)));
    }

    pub fn get_scrapers(&self) -> &[Arc<Mutex<ScraperType>>] {
        &self.scrapers
    }

    pub fn get_scraper_for_url(&self, url: &str) -> Result<ScraperType> {
        for scraper in &self.scrapers {
            let s = scraper.lock().unwrap();
            if s.can_handle(url) {
                let cloned = s.clone();
                drop(s);
                return Ok(cloned);
            }
        }
        Err(Error::Scraping(format!("No scraper found for URL: {}", url)))
    }

    pub fn get_scrapers_for_source(&self, source: &str) -> Result<Vec<ScraperType>> {
        let (country, name) = self.parse_source(source)?;
        let mut result = Vec::new();
        
        if let Some(name) = name {
            // Get specific scraper
            if let Ok(scraper) = self.get_scraper(&country, &name) {
                result.push(scraper);
            }
        } else {
            // Get all scrapers for country
            if let Some(scrapers) = self.get_all_scrapers().get(&country) {
                for scraper in scrapers {
                    let s = scraper.lock().unwrap();
                    let cloned = s.clone();
                    drop(s);
                    result.push(cloned);
                }
            }
        }
        
        Ok(result)
    }

    pub async fn scrape_url(&mut self, url: &str) -> Result<(Article, ArticleStatus)> {
        let mut scraper = self.get_scraper_for_url(url)?;
        let article = scraper.scrape_article(url).await?;
        let existing_articles = self.storage.get_by_source(scraper.source_metadata().name).await?;
        
        let status = if let Some(existing) = existing_articles.iter().find(|a| a.url == url) {
            if existing.content == article.content {
                ArticleStatus::Unchanged
            } else {
                ArticleStatus::Updated
            }
        } else {
            ArticleStatus::New
        };

        if matches!(status, ArticleStatus::New | ArticleStatus::Updated) {
            let embedding = self.inference.generate_embeddings(&article.content).await?;
            self.storage.store_article(&article, &embedding).await?;
        }

        Ok((article, status))
    }

    pub async fn scrape_all(&mut self) -> Result<Vec<(Article, ArticleStatus)>> {
        let mut results = Vec::new();
        let mut logger = crate::logging::init_logging();

        // Get all scrapers
        let scraper_sources: Vec<String> = self.scrapers.iter()
            .map(|s| s.lock().unwrap().source_metadata().name.to_string())
            .collect();

        // For each scraper
        for source in scraper_sources {
            // Find the scraper again to get a mutable reference
            if let Some(scraper) = self.scrapers.iter_mut().find(|s| s.lock().unwrap().source_metadata().name == source) {
                // Get all article URLs
                let urls = scraper.lock().unwrap().get_article_urls().await?;
                let metadata = scraper.lock().unwrap().source_metadata();
                let prefix = format!("{} {} |", metadata.region.emoji, metadata.emoji);
                logger = logger.with_new_prefixes(prefix);
                logger.info(&format!("Found {} articles", urls.len()));

                // Process articles in batches of 5
                let batch_size = 5;
                for chunk in urls.chunks(batch_size) {
                    let mut batch_results = Vec::new();
                    
                    // Process each URL in the batch
                    for url in chunk {
                        match self.scrape_url(url).await {
                            Ok(result) => {
                                let (article, status) = result.clone();
                                let status_emoji = match status {
                                    ArticleStatus::New => "💥",
                                    ArticleStatus::Updated => "👻",
                                    ArticleStatus::Unchanged => "✅",
                                };
                                let authors_str = if article.authors.is_empty() {
                                    logger.debug("🤷🏾‍♂️ No authors found");
                                    String::from("🤷🏾‍♂️ ")
                                } else {
                                    format!(" | by \x1b[1m{}\x1b[0m", article.authors.join(", "))
                                };
                                let message = format!("{} {} - {}{}", 
                                    status_emoji, 
                                    article.title,
                                    authors_str,
                                    if matches!(status, ArticleStatus::New | ArticleStatus::Updated) {
                                        " (saved)"
                                    } else {
                                        " (unchanged)"
                                    }
                                );
                                logger.info(&message);
                                batch_results.push(result);
                            }
                            Err(e) => {
                                logger.error(&format!("Failed to scrape {}: {}", url, e));
                            }
                        }
                    }
                    results.extend(batch_results);
                }
            }
        }

        Ok(results)
    }

    fn parse_source(&self, source: &str) -> Result<(String, Option<String>)> {
        let parts: Vec<&str> = source.split('/').collect();
        if parts.len() > 2 {
            return Err(Error::Scraping(
                "Invalid source format. Expected: country or country/source".to_string(),
            ));
        }
        Ok((parts[0].to_string(), parts.get(1).map(|s| s.to_string())))
    }

    fn get_all_scrapers(&self) -> HashMap<String, Vec<Arc<Mutex<ScraperType>>>> {
        let mut scrapers = HashMap::new();
        scrapers.insert("argentina".to_string(), crate::scrapers::argentina::get_scrapers());
        scrapers
    }

    fn get_scraper(&self, country: &str, name: &str) -> Result<ScraperType> {
        let all_scrapers = self.get_all_scrapers();
        
        if let Some(scrapers) = all_scrapers.get(country) {
            for scraper in scrapers {
                let s = scraper.lock().unwrap();
                if s.source_metadata().name.to_lowercase().replace('í', "i") == name.to_lowercase().replace('í', "i") 
                   || s.cli_names().contains(&name) {
                    let cloned = s.clone();
                    drop(s);
                    return Ok(cloned);
                }
            }
            
            Err(Error::Scraping(format!("Scraper not found: {}/{}", country, name)))
        } else {
            Err(Error::Scraping(format!(
                "Country not supported: {}",
                country
            )))
        }
    }

    pub async fn list_scrapers(&self) -> Result<()> {
        let mut logger = crate::logging::init_logging();
        tracing::info!("Available scrapers:");
        for scraper in self.get_scrapers() {
            let s = scraper.lock().unwrap();
            let metadata = s.source_metadata();
            let prefix = format!("{} {} {}", metadata.region.emoji, metadata.emoji, metadata.name);
            logger = logger.with_prefix(prefix);
            logger.info(&metadata.region.name);
        }
        Ok(())
    }
}

#[async_trait]
impl ArticleStorage for ScraperManager {
    async fn store_article(&self, article: &Article, embedding: &[f32]) -> Result<()> {
        self.storage.store_article(article, embedding).await
    }

    async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Article>> {
        self.storage.find_similar(embedding, limit).await
    }

    async fn get_by_source(&self, source: &str) -> Result<Vec<Article>> {
        self.storage.get_by_source(source).await
    }
} 