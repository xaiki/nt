use clap::Parser;
use nt_core::{Result, ArticleStorage, Article, Scraper};
use nt_storage::{StorageBackend, UrlConfig};
use chrono::Utc;
use nt_scrappers::cli::{ScraperArgs, ScraperCommands as NtScraperCommands, handle_command};
use std::sync::Arc;
use tracing::info;
use std::str::FromStr;
use std::time::Duration;
use nt_scrappers::ScraperManager;
use nt_scrappers::scrapers::argentina::ClarinScraper;

const DEFAULT_VECTOR_SIZE: u64 = 768;

#[derive(Debug, Clone)]
struct HumanDuration(Duration);

impl FromStr for HumanDuration {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut total_seconds = 0u64;
        let mut current_number = String::new();
        let mut has_unit = false;
        
        for c in s.chars() {
            if c.is_ascii_digit() {
                current_number.push(c);
            } else if let Some(num) = current_number.parse::<u64>().ok() {
                match c {
                    's' => total_seconds += num,
                    'm' => total_seconds += num * 60,
                    'h' => total_seconds += num * 3600,
                    'd' => total_seconds += num * 86400,
                    _ => return Err(format!("Invalid duration unit: {}", c)),
                }
                current_number.clear();
                has_unit = true;
            } else if !c.is_whitespace() {
                return Err(format!("Invalid character in duration: {}", c));
            }
        }

        // If we have a number but no unit, assume seconds
        if !current_number.is_empty() {
            if let Ok(num) = current_number.parse::<u64>() {
                total_seconds += num;
                has_unit = true;
            } else {
                return Err("Invalid number in duration".to_string());
            }
        }

        if !has_unit {
            return Err("Duration must include a number".to_string());
        }

        Ok(HumanDuration(Duration::from_secs(total_seconds)))
    }
}

async fn check_storage(storage: &Arc<dyn ArticleStorage>, storage_type: &str) -> Result<()> {
    let test_article = Article {
        url: "http://test.com".to_string(),
        title: "Test Article".to_string(),
        content: "Test content".to_string(),
        published_at: Utc::now(),
        source: "test".to_string(),
        sections: vec![],
        summary: None,
        authors: vec!["Test Author".to_string()],
        related_articles: Vec::new(),
    };

    storage.store_article(&test_article, &vec![0.0; DEFAULT_VECTOR_SIZE as usize]).await?;
    
    // Try to retrieve it
    let articles = storage.get_by_source("test").await?;
    if !articles.iter().any(|a| a.url == test_article.url) {
        return Err(nt_core::Error::Storage("Failed to retrieve test article".to_string()));
    }

    info!("🏦 Storage backend initialized successfully (using {})", storage_type);

    // Clean up test article
    if let Err(e) = storage.delete_article(&test_article.url).await {
        info!("⚠️ Failed to clean up test article: {}", e);
    }

    Ok(())
}

async fn check_storage_with_retry(storage: &Arc<dyn ArticleStorage>, storage_type: &str, max_retries: u32, timeout: Duration) -> Result<()> {
    let mut retries = 0;
    let mut last_error = None;

    while retries < max_retries {
        match tokio::time::timeout(timeout, check_storage(storage, storage_type)).await {
            Ok(result) => return result,
            Err(timeout_error) => {
                last_error = Some(nt_core::Error::Storage(format!("Storage health check timed out: {}", timeout_error)));
                retries += 1;
                if retries < max_retries {
                    info!("Storage health check failed, retrying {}/{}...", retries, max_retries);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| nt_core::Error::Storage("Storage health check failed after all retries".to_string())))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long, default_value = "memory")]
    storage: String,
    #[arg(long)]
    model_url: Option<String>,
    #[arg(long)]
    backend_url: Option<String>,
    #[arg(long, default_value = "ollama", help = "Model to use for inference. Available models: ollama (default), deepseek")]
    model: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    Scrape {
        #[command(subcommand)]
        command: Option<ScraperCommands>,
    },
}

#[derive(clap::Subcommand, Debug)]
enum ScraperCommands {
    Source {
        /// The source to scrape in format country/source (e.g. argentina/clarin). If not specified, scrapes all sources.
        #[arg(required = false)]
        source: Option<String>,
        /// Run in periodic mode with the specified interval (e.g. 1h, 30m, 1d, 1h15m30s)
        #[arg(long, default_value = "1h")]
        interval: Option<HumanDuration>,
    },
    List,
    Url {
        url: String,
    },
}

async fn create_storage<T: StorageBackend + ArticleStorage + 'static>(backend_url: Option<&str>) -> Result<Arc<dyn ArticleStorage>> {
    let mut retries = 3;
    let mut last_error = None;
    let storage_type = std::any::type_name::<T>().split("::").last().unwrap_or("unknown").to_string();

    while retries > 0 {
        match T::new().await {
            Ok(mut storage) => {
                if let Some(url) = backend_url {
                    if let Some(config) = storage.get_config() {
                        config.with_url(url);
                    }
                }
                let storage = Arc::new(storage) as Arc<dyn ArticleStorage>;
                // Check storage health with retries
                if let Err(e) = check_storage_with_retry(&storage, &storage_type, 3, Duration::from_secs(10)).await {
                    last_error = Some(e);
                    retries -= 1;
                    if retries > 0 {
                        info!("Storage initialization failed, retrying {}/3...", 4 - retries);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                } else {
                    return Ok(storage);
                }
            }
            Err(e) => {
                last_error = Some(e);
                retries -= 1;
                if retries > 0 {
                    info!("Storage initialization failed, retrying {}/3...", 4 - retries);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| nt_core::Error::Storage("Storage initialization failed after all retries".to_string())))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let storage: Arc<dyn ArticleStorage> = nt_storage::create_storage(
        cli.storage.as_str(),
        cli.backend_url.as_deref()
    ).await?;

    // Check storage connection
    info!("💾 Checking storage connection...");
    check_storage(&storage, cli.storage.as_str()).await?;
    info!("✨ Storage initialized successfully (using {})", cli.storage);

    // Initialize inference model based on configuration
    let inference_config = nt_inference::InferenceConfig {
        model_url: cli.model_url,
        backend_url: cli.backend_url,
    };
    let config = nt_inference::Config {
        api_key: None,
        model_name: Some(cli.model.clone()),
        backend_config: nt_storage::backends::memory::MemoryConfig::new().config,
        inference_config,
    };
    let inference = nt_inference::models::create_model(Some(config)).await?;
    info!("🧠 Inference model initialized successfully (using {})", inference.name());
    let mut manager = ScraperManager::new(storage.clone(), inference.clone()).await?;
    
    // Add all available scrapers
    let mut scraper_names = Vec::new();
    for factory in nt_scrappers::scrapers::get_scraper_factories() {
        let scraper = factory();
        scraper_names.push(scraper.source_metadata().name);
        manager.add_scraper_factory(factory);
    }
    info!("🦗 Scrapers initialized successfully: {}", scraper_names.join(", "));

    match cli.command {
        Commands::Scrape { command } => match command.unwrap_or(ScraperCommands::Source { source: None, interval: None }) {
            ScraperCommands::Source { source, interval } => {
                info!("🦗 Scraping articles from {}", if source.is_none() || source.as_ref().unwrap().is_empty() { "all sources" } else { source.as_ref().unwrap() });
                let args = ScraperArgs {
                    command: NtScraperCommands::Source { source: source.map(|s| s.to_string()) },
                };
                
                if let Some(interval) = interval {
                    info!("Running in periodic mode with {} interval", interval.0.as_secs());
                    loop {
                        info!("Starting scrape cycle");
                        if let Err(e) = handle_command(args.clone(), &mut manager).await {
                            eprintln!("Error during scrape: {}", e);
                        }
                        info!("Waiting {}s before next scrape", interval.0.as_secs());
                        tokio::time::sleep(interval.0).await;
                    }
                } else {
                    handle_command(args, &mut manager).await?;
                }
            }
            ScraperCommands::List => {
                let args = ScraperArgs {
                    command: NtScraperCommands::List,
                };
                handle_command(args, &mut manager).await?;
            }
            ScraperCommands::Url { url } => {
                info!("Scraping single URL: {}", url);
                let args = ScraperArgs {
                    command: NtScraperCommands::Url { url: url.clone() },
                };
                handle_command(args, &mut manager).await?;
            }
        },
    }

    // Create a test article
    let test_article = Article {
        url: "http://test.com".to_string(),
        title: "Test Article".to_string(),
        content: "This is a test article.".to_string(),
        published_at: chrono::Utc::now(),
        source: "test".to_string(),
        sections: vec![],
        summary: None,
        authors: vec!["Test Author".to_string()],
        related_articles: Vec::new(),
    };

    // Try to store the article with a test embedding
    storage.store_article(&test_article, &vec![0.0; DEFAULT_VECTOR_SIZE as usize]).await?;
    
    // Try to retrieve it
    let similar = storage.find_similar(&vec![0.0; DEFAULT_VECTOR_SIZE as usize], 1).await?;
    println!("Found {} similar articles", similar.len());

    Ok(())
} 