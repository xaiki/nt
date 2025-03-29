use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::ops::Deref;
use nt_core::{Article, Result, Error, ArticleStorage, InferenceModel, ArticleStatus, Scraper};
use crate::scrapers::ScraperType;
use tracing::info;
use tokio::sync::{Mutex as TokioMutex, Semaphore, mpsc};
use futures::future::join_all;
use std::sync::Mutex as StdMutex;
use tokio::task::JoinHandle;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub struct ScraperManager {
    storage: Arc<dyn ArticleStorage>,
    inference: Arc<dyn InferenceModel>,
    scrapers: Arc<StdMutex<Vec<Arc<Mutex<ScraperType>>>>>,
    semaphore: Arc<Semaphore>,
    inference_tasks: Arc<TokioMutex<Vec<JoinHandle<Result<()>>>>>,
    progress: Arc<MultiProgress>,
}

impl ScraperManager {
    pub async fn new(storage: Arc<dyn ArticleStorage>, inference: Arc<dyn InferenceModel>) -> Result<Self> {
        Ok(Self {
            storage,
            inference,
            scrapers: Arc::new(StdMutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(32)), // Limit concurrent operations to 32
            inference_tasks: Arc::new(TokioMutex::new(Vec::new())),
            progress: Arc::new(MultiProgress::new()),
        })
    }

    pub fn add_scraper(&self, scraper: ScraperType) {
        self.scrapers.lock().unwrap().push(Arc::new(Mutex::new(scraper)));
    }

    pub fn get_scrapers(&self) -> Vec<Arc<Mutex<ScraperType>>> {
        self.scrapers.lock().unwrap().clone()
    }

    pub fn get_scraper_for_url(&self, url: &str) -> Result<ScraperType> {
        for scraper in self.get_scrapers() {
            let s = scraper.lock().unwrap();
            if s.can_handle(url) {
                return Ok(s.clone());
            }
        }
        Err(nt_core::Error::Scraping(format!("No scraper found for URL: {}", url)))
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
            
            // Process similar articles in parallel
            let similar_futures: Vec<_> = similar_articles.into_iter()
                .filter(|a| a.url != article.url)
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
            for _ in 0..article.related_articles.len() {
                emoji_chain.push_str("📊");
            }

            // Store the processed article
            storage.store_article(&article, &article_embedding).await?;
            emoji_chain.push_str("✨");
            emoji_chain.push_str("✅");

            info!("{} Article processed: {}", emoji_chain, article.title);
            Ok(())
        });

        let mut tasks = self.inference_tasks.lock().await;
        tasks.push(handle);
    }

    pub async fn scrape_url(&self, url: &str) -> Result<Article> {
        let mut scraper = self.get_scraper_for_url(url)?;
        let article = scraper.scrape_article(url).await?;
        self.queue_inference_task(article.clone()).await;
        Ok(article)
    }

    pub async fn scrape_source(&self, source: Option<&str>) -> Result<Vec<Article>> {
        let mut articles = Vec::new();
        
        // If source is specified, get specific scrapers
        let scrapers = if let Some(target_source) = source {
            self.get_scrapers_for_source(target_source)?
        } else {
            // Convert Arc<Mutex<ScraperType>> to ScraperType
            self.get_scrapers().into_iter()
                .map(|s| {
                    let s = s.lock().unwrap();
                    s.clone()
                })
                .collect()
        };
        
        // Create progress bar for scrapers
        let scraper_pb = self.progress.add(ProgressBar::new(scrapers.len() as u64));
        scraper_pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed}] {bar:40} {pos:>3}/{len:3} {msg}")
            .unwrap());
        scraper_pb.set_message("Scraping sources");
        
        // Run all scrapers in parallel
        let scraper_futures: Vec<_> = scrapers.into_iter().map(|scraper| {
            let source_name = scraper.source_metadata().name;
            let scraper_pb = scraper_pb.clone();
            
            async move {
                scraper_pb.set_message(format!("Scraping: {}", source_name));
                let urls = scraper.get_article_urls().await?;
                let mut scraper_articles = Vec::new();
                
                // Process URLs in parallel
                let url_futures: Vec<_> = urls.into_iter().map(|url| {
                    let manager = self.clone();
                    async move {
                        match manager.scrape_url(&url).await {
                            Ok(article) => Some(article),
                            Err(e) => {
                                info!("Failed to scrape {}: {}", url, e);
                                None
                            }
                        }
                    }
                }).collect();

                let results = join_all(url_futures).await;
                for result in results {
                    if let Some(article) = result {
                        scraper_articles.push(article);
                    }
                }
                
                scraper_pb.inc(1);
                Ok::<_, nt_core::Error>(scraper_articles)
            }
        }).collect();

        // Collect results from all scrapers
        let results = join_all(scraper_futures).await;
        for result in results {
            if let Ok(scraper_articles) = result {
                articles.extend(scraper_articles);
            }
        }

        // Wait for all inference tasks to complete
        let mut tasks = self.inference_tasks.lock().await;
        
        // Process tasks in chunks to avoid overwhelming the system
        while !tasks.is_empty() {
            let mut chunk = Vec::new();
            for _ in 0..32.min(tasks.len()) {
                if let Some(task) = tasks.pop() {
                    chunk.push(task);
                }
            }
            
            // Wait for current chunk to complete
            for handle in chunk {
                if let Err(e) = handle.await {
                    info!("Inference task failed: {}", e);
                }
            }
        }

        // Wait for all progress bars to complete
        (*self.progress).clear();

        Ok(articles)
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

    async fn delete_article(&self, url: &str) -> Result<()> {
        self.storage.delete_article(url).await
    }

    async fn get_article_embedding(&self, url: &str) -> Result<Vec<f32>> {
        self.storage.get_article_embedding(url).await
    }
} 