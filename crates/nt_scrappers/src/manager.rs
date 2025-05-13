use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::ops::Deref;
use nt_core::{Article, Result, Error, ArticleStorage, InferenceModel, ArticleStatus, Scraper};
use crate::scrapers::ScraperType;
use log::info;
use tokio::sync::{Mutex as TokioMutex, Semaphore, mpsc};
use futures::future::join_all;
use std::sync::Mutex as StdMutex;
use tokio::task::JoinHandle;
use std::io::{stderr, IsTerminal};
use anyhow;
use std::io::Write;
use nt_progress::{ProgressDisplay, ThreadLogger, Config, ThreadMode, TaskHandle};
use std::time::Duration;
use tokio::time::sleep;
use nt_core::ArticleSection;
use crate::scrapers::{ScraperFactory, get_scraper_factories};

type BoxedScraper = Box<dyn Scraper + Send + Sync>;

pub struct ScraperManager {
    storage: Arc<dyn ArticleStorage>,
    inference: Arc<dyn InferenceModel>,
    factories: Vec<ScraperFactory>,
    semaphore: Arc<Semaphore>,
    inference_tasks: Arc<TokioMutex<Vec<JoinHandle<Result<()>>>>>,
}

impl ScraperManager {
    pub async fn new(storage: Arc<dyn ArticleStorage>, inference: Arc<dyn InferenceModel>) -> Result<Self> {
        Ok(Self {
            storage,
            inference,
            factories: get_scraper_factories(),
            semaphore: Arc::new(Semaphore::new(10)),
            inference_tasks: Arc::new(TokioMutex::new(Vec::new())),
        })
    }

    pub fn add_scraper_factory(&mut self, factory: ScraperFactory) {
        self.factories.push(factory);
    }

    pub fn get_scrapers(&self) -> Vec<BoxedScraper> {
        self.factories.iter().map(|f| f()).collect()
    }

    pub fn get_scraper_for_url(&self, url: &str) -> Result<BoxedScraper> {
        for factory in &self.factories {
            let scraper = factory();
            if scraper.can_handle(url) {
                return Ok(scraper);
            }
        }
        Err(nt_core::Error::Scraping(format!("No scraper found for URL: {}", url)))
    }

    pub fn get_scrapers_for_source(&self, source: &str) -> Result<Vec<BoxedScraper>> {
        let (country, name) = self.parse_source(source)?;
        let mut result = Vec::new();
        for factory in &self.factories {
            let scraper = factory();
            let meta = scraper.source_metadata();
            if meta.region.name == country {
                if let Some(ref name) = name {
                    if scraper.cli_names().contains(&name.as_str()) {
                        result.push(scraper);
                    }
                } else {
                    result.push(scraper);
                }
            }
        }
        Ok(result)
    }

    async fn process_article(&self, mut article: Article) -> Result<()> {
        info!("📰 Processing article: {}", article.title);
        
        // Generate article summary
        info!("🤖 Generating summary for article: {}", article.title);
        let _permit = self.semaphore.acquire().await.map_err(|e| nt_core::Error::External(e.into()))?;
        article.summary = Some(self.inference.summarize_article(&article).await?);
        info!("✨ Summary generated successfully: {:?}", article.summary);

        // Generate section summaries and embeddings in parallel
        let num_sections = article.sections.len();
        info!("📑 Processing {} sections", num_sections);
        
        let section_futures: Vec<_> = article.sections.iter_mut().enumerate().map(|(i, section)| {
            let inference = self.inference.clone();
            let semaphore = self.semaphore.clone();
            async move {
                info!("📝 Processing section {}/{}", i + 1, num_sections);
                
                // Generate section summary
                info!("🤖 Generating summary for section {}", i + 1);
                let _permit = semaphore.acquire().await.map_err(|e| nt_core::Error::External(e.into()))?;
                section.summary = Some(inference.summarize_sections(&[section.clone()]).await?[0].clone());
                info!("✨ Section summary generated: {:?}", section.summary);
                
                // Generate section embedding
                info!("🔢 Generating embedding for section {}", i + 1);
                let _permit = semaphore.acquire().await.map_err(|e| nt_core::Error::External(e.into()))?;
                section.embedding = Some(inference.generate_embeddings(&section.content).await?);
                info!("✨ Section embedding generated");
                
                Ok::<_, nt_core::Error>(())
            }
        }).collect();

        join_all(section_futures).await.into_iter().collect::<Result<Vec<_>>>()?;

        // Generate article embedding for similarity search
        info!("🔢 Generating article embedding for similarity search");
        let _permit = self.semaphore.acquire().await.map_err(|e| nt_core::Error::External(e.into()))?;
        let article_embedding = self.inference.generate_embeddings(&article.content).await?;
        info!("✨ Article embedding generated");

        // Find similar articles
        info!("🔍 Finding similar articles");
        let similar_articles = self.storage.find_similar(&article_embedding, 5).await?;
        info!("✨ Found {} similar articles", similar_articles.len());
        
        // Process similar articles in parallel
        info!("🔄 Converting similar articles to related articles");
        let similar_futures: Vec<_> = similar_articles.into_iter()
            .filter(|a| a.url != article.url) // Exclude self
            .map(|a| {
                let inference = self.inference.clone();
                let semaphore = self.semaphore.clone();
                let article_embedding = article_embedding.clone();
                async move {
                    info!("📊 Calculating similarity score for article: {}", a.title);
                    let _permit = semaphore.acquire().await.map_err(|e| nt_core::Error::External(e.into()))?;
                    let a_embedding = inference.generate_embeddings(&a.content).await?;
                    let similarity = nt_core::cosine_similarity(&article_embedding, &a_embedding);
                    info!("✨ Similarity score calculated: {:.2}", similarity);
                    let related = nt_core::RelatedArticle {
                        article: a,
                        similarity_score: Some(similarity),
                    };
                    Ok::<_, nt_core::Error>(related)
                }
            })
            .collect();

        article.related_articles = join_all(similar_futures).await.into_iter().collect::<Result<Vec<_>>>()?;
        info!("✨ Related articles processed");

        // Store the processed article
        info!("💾 Storing processed article");
        self.storage.store_article(&article, &article_embedding).await?;
        info!("✨ Article stored successfully");

        info!("✅ Article processing completed: {}", article.title);
        Ok(())
    }

    async fn queue_inference_task(&self, article: Article) {
        let inference = self.inference.clone();
        let storage = self.storage.clone();
        let semaphore = self.semaphore.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.map_err(|e| nt_core::Error::External(e.into()))?;
            
            let mut article = article;
            let mut emoji_chain = String::new();
            
            // Generate article summary
            article.summary = Some(inference.summarize_article(&article).await?);
            emoji_chain.push_str("🤖");

            // Generate section summaries and embeddings in parallel
            let num_sections = article.sections.len();
            let section_futures: Vec<_> = article.sections.iter_mut().enumerate().map(|(i, section)| {
                let inference = inference.clone();
                let semaphore = semaphore.clone();
                async move {
                    let _permit = semaphore.acquire().await.map_err(|e| nt_core::Error::External(e.into()))?;
                    section.summary = Some(inference.summarize_sections(&[section.clone()]).await?[0].clone());
                    section.embedding = Some(inference.generate_embeddings(&section.content).await?);
                    Ok::<_, nt_core::Error>(())
                }
            }).collect();

            join_all(section_futures).await.into_iter().collect::<Result<Vec<_>>>()?;
            emoji_chain.push_str("📑");

            // Generate article embedding for similarity search
            let article_embedding = inference.generate_embeddings(&article.content).await?;
            emoji_chain.push_str("🔢");

            // Find similar articles
            let similar_articles = storage.find_similar(&article_embedding, 5).await?;
            emoji_chain.push_str("🔍");

            // Process similar articles in parallel
            let similar_futures: Vec<_> = similar_articles.into_iter()
                .filter(|a| a.url != article.url) // Exclude self
                .map(|a| {
                    let inference = inference.clone();
                    let semaphore = semaphore.clone();
                    let article_embedding = article_embedding.clone();
                    async move {
                        let _permit = semaphore.acquire().await.map_err(|e| nt_core::Error::External(e.into()))?;
                        let a_embedding = inference.generate_embeddings(&a.content).await?;
                        let similarity = nt_core::cosine_similarity(&article_embedding, &a_embedding);
                        let related = nt_core::RelatedArticle {
                            article: a,
                            similarity_score: Some(similarity),
                        };
                        Ok::<_, nt_core::Error>(related)
                    }
                })
                .collect();

            article.related_articles = join_all(similar_futures).await.into_iter().collect::<Result<Vec<_>>>()?;
            emoji_chain.push_str("🔄");

            // Store the processed article
            storage.store_article(&article, &article_embedding).await?;
            emoji_chain.push_str("💾");

            Ok::<_, nt_core::Error>(())
        });

        self.inference_tasks.lock().await.push(handle);
    }

    pub async fn scrape_url(&self, url: &str) -> Result<Article> {
        let mut scraper = self.get_scraper_for_url(url)?;
        scraper.scrape_article(url).await
    }

    pub async fn scrape_source(&self, source: Option<&str>) -> Result<Vec<Article>> {
        let mut articles = Vec::new();
        let mut progress = None;

        if let Some(source) = source {
            let scrapers = self.get_scrapers_for_source(source)?;
            for mut scraper in scrapers {
                let urls = scraper.get_article_urls().await?;
                
                if progress.is_none() {
                    progress = Some(ProgressDisplay::new_with_mode(ThreadMode::Window(3)).await?);
                }

                let progress_handle = progress.as_ref().unwrap().clone();
                let url_futures: Vec<_> = urls.into_iter().enumerate().map(|(_j, url)| {
                    let progress = progress_handle.clone();
                    async move {
                        let article = self.scrape_url(&url).await?;
                        progress.update_progress(0).await?;
                        Ok::<_, nt_core::Error>(article)
                    }
                }).collect();

                let mut scraped_articles = join_all(url_futures).await.into_iter().collect::<Result<Vec<_>>>()?;
                articles.append(&mut scraped_articles);
            }
        } else {
            // Scrape all sources
            let all_scrapers = self.get_all_scrapers();
            for (_country, scrapers) in all_scrapers {
                for mut scraper in scrapers {
                    let urls = scraper.get_article_urls().await?;
                    
                    if progress.is_none() {
                        progress = Some(ProgressDisplay::new_with_mode(ThreadMode::Window(3)).await?);
                    }

                    let progress_handle = progress.as_ref().unwrap().clone();
                    let url_futures: Vec<_> = urls.into_iter().enumerate().map(|(_j, url)| {
                        let progress = progress_handle.clone();
                        async move {
                            let article = self.scrape_url(&url).await?;
                            progress.update_progress(0).await?;
                            Ok::<_, nt_core::Error>(article)
                        }
                    }).collect();

                    let mut scraped_articles = join_all(url_futures).await.into_iter().collect::<Result<Vec<_>>>()?;
                    articles.append(&mut scraped_articles);
                }
            }
        }

        Ok(articles)
    }

    fn parse_source(&self, source: &str) -> Result<(String, Option<String>)> {
        let parts: Vec<&str> = source.split('/').collect();
        match parts.len() {
            1 => Ok((parts[0].to_string(), None)),
            2 => Ok((parts[0].to_string(), Some(parts[1].to_string()))),
            _ => Err(nt_core::Error::Scraping(format!("Invalid source format: {}", source))),
        }
    }

    fn get_all_scrapers(&self) -> HashMap<String, Vec<BoxedScraper>> {
        let mut scrapers = HashMap::new();
        for factory in &self.factories {
            let scraper = factory();
            let meta = scraper.source_metadata();
            scrapers.entry(meta.region.name.to_string()).or_insert_with(Vec::new).push(scraper);
        }
        scrapers
    }

    fn get_scraper(&self, country: &str, name: &str) -> Result<BoxedScraper> {
        for factory in &self.factories {
            let scraper = factory();
            let meta = scraper.source_metadata();
            if meta.region.name == country && scraper.cli_names().contains(&name) {
                return Ok(scraper);
            }
        }
        Err(nt_core::Error::Scraping(format!("No scraper found for {}/{}", country, name)))
    }

    pub async fn list_scrapers(&self) -> Result<()> {
        let all_scrapers = self.get_all_scrapers();
        
        for (country, scrapers) in all_scrapers {
            println!("{}:", country);
            for scraper in scrapers {
                let metadata = scraper.source_metadata();
                println!("  - {} ({})", metadata.name, metadata.region.name);
            }
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

    async fn delete_article(&self, url: &str) -> Result<()> {
        self.storage.delete_article(url).await
    }

    async fn get_article_embedding(&self, url: &str) -> Result<Vec<f32>> {
        self.storage.get_article_embedding(url).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scrapers::argentina::ClarinScraper;
    use std::sync::Arc;

    struct MockStorage;
    struct MockInference;

    #[async_trait]
    impl ArticleStorage for MockStorage {
        async fn store_article(&self, _article: &Article, _embedding: &[f32]) -> Result<()> {
            Ok(())
        }

        async fn find_similar(&self, _embedding: &[f32], _limit: usize) -> Result<Vec<Article>> {
            Ok(Vec::new())
        }

        async fn get_by_source(&self, _source: &str) -> Result<Vec<Article>> {
            Ok(Vec::new())
        }

        async fn delete_article(&self, _url: &str) -> Result<()> {
            Ok(())
        }

        async fn get_article_embedding(&self, _url: &str) -> Result<Vec<f32>> {
            Ok(Vec::new())
        }
    }

    #[async_trait]
    impl InferenceModel for MockInference {
        fn name(&self) -> &str {
            "mock"
        }

        async fn summarize_article(&self, _article: &Article) -> Result<String> {
            Ok("Test summary".to_string())
        }

        async fn summarize_sections(&self, _sections: &[ArticleSection]) -> Result<Vec<String>> {
            Ok(vec!["Test section summary".to_string()])
        }

        async fn generate_embeddings(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.0; 384])
        }
    }

    #[tokio::test]
    async fn test_scrape_url() {
        let storage = Arc::new(MockStorage);
        let inference = Arc::new(MockInference);
        let mut manager = ScraperManager::new(storage, inference).await.unwrap();
        // Add a scraper factory for ClarinScraper
        manager.add_scraper_factory(Box::new(|| Box::new(ClarinScraper::new())));
        
        // Get article URLs
        let urls = manager.scrape_source(Some("argentina/clarin")).await.unwrap();
        assert!(!urls.is_empty(), "No articles found");
        
        // Try to scrape the first article
        let article = &urls[0];
        println!("Scraping article: {}", article.url);
        
        let result = manager.scrape_url(&article.url).await;
        assert!(result.is_ok());
        
        let article = result.unwrap();
        assert!(!article.title.is_empty());
        assert!(!article.content.is_empty());
        assert!(!article.sections.is_empty());
    }
} 