use clap::Parser;
use nt_core::{Result, storage::ArticleStorage, Article};
use nt_storage::{InMemoryStorage, StorageBackend};
use chrono::Utc;
use nt_scrappers::cli::{ScraperArgs, ScraperCommands as NtScraperCommands, handle_command};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use std::str::FromStr;
use std::time::Duration;

#[cfg(feature = "chroma")]
use nt_storage::ChromaDBStorage;

#[cfg(feature = "qdrant")]
use nt_storage::QdrantStorage;

#[cfg(feature = "sqlite")]
use nt_storage::SQLiteStorage;

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

async fn check_storage(storage: &Arc<RwLock<dyn ArticleStorage>>) -> Result<()> {
    let test_article = Article {
        url: "http://test.com".to_string(),
        title: "Test Article".to_string(),
        content: "Test content".to_string(),
        published_at: Utc::now(),
        source: "test".to_string(),
        sections: vec![],
        summary: None,
        authors: vec!["Test Author".to_string()],
    };

    // Try to store the article
    storage.write().await.store_article(&test_article).await?;
    
    // Try to retrieve it
    let articles = storage.read().await.get_by_source("test").await?;
    if !articles.iter().any(|a| a.url == test_article.url) {
        return Err(nt_core::Error::Storage("Failed to retrieve test article".to_string()));
    }

    Ok(())
}

async fn check_storage_with_retry(storage: &Arc<RwLock<dyn ArticleStorage>>, max_retries: u32, timeout: Duration) -> Result<()> {
    let mut retries = 0;
    let mut last_error = None;

    while retries < max_retries {
        match tokio::time::timeout(timeout, check_storage(storage)).await {
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

async fn create_storage<T: StorageBackend + 'static>() -> Result<Arc<RwLock<dyn ArticleStorage>>> {
    let storage = T::new().await?;
    Ok(Arc::new(RwLock::new(storage)))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let storage = match cli.storage.as_str() {
        "memory" => create_storage::<InMemoryStorage>().await?,
        #[cfg(feature = "chroma")]
        "chroma" => create_storage::<ChromaDBStorage>().await?,
        #[cfg(feature = "qdrant")]
        "qdrant" => create_storage::<QdrantStorage>().await?,
        #[cfg(feature = "sqlite")]
        "sqlite" => create_storage::<SQLiteStorage>().await?,
        _ => {
            #[allow(unused_mut)]
            let mut msg = "Unknown storage backend. Available backends: memory".to_string();
            #[cfg(feature = "chroma")]
            msg.push_str(", chroma");
            #[cfg(feature = "qdrant")]
            msg.push_str(", qdrant");
            #[cfg(feature = "sqlite")]
            msg.push_str(", sqlite");
            eprintln!("{}", msg);
            return Err(nt_core::Error::Storage(msg));
        }
    };

    // Check storage health with retries before proceeding
    if let Err(e) = check_storage_with_retry(&storage, 3, Duration::from_secs(10)).await {
        eprintln!("Storage health check failed after all retries: {}", e);
        return Err(e);
    }

    match cli.command {
        Commands::Scrape { command } => match command.unwrap_or(ScraperCommands::Source { source: None, interval: None }) {
            ScraperCommands::Source { source, interval } => {
                info!("Scraping articles from {}", if source.is_none() || source.as_ref().unwrap().is_empty() { "all sources" } else { source.as_ref().unwrap() });
                let args = ScraperArgs {
                    command: NtScraperCommands::Source { source: source.map(|s| s.to_string()) },
                };
                let storage_guard = storage.read().await;
                
                if let Some(interval) = interval {
                    info!("Running in periodic mode with {} interval", interval.0.as_secs());
                    loop {
                        info!("Starting scrape cycle");
                        if let Err(e) = handle_command(args.clone(), &*storage_guard).await {
                            eprintln!("Error during scrape: {}", e);
                        }
                        info!("Waiting {}s before next scrape", interval.0.as_secs());
                        tokio::time::sleep(interval.0).await;
                    }
                } else {
                    handle_command(args, &*storage_guard).await?;
                }
            }
            ScraperCommands::List => {
                let args = ScraperArgs {
                    command: NtScraperCommands::List,
                };
                let storage_guard = storage.read().await;
                handle_command(args, &*storage_guard).await?;
            }
            ScraperCommands::Url { url } => {
                info!("Scraping single URL: {}", url);
                let args = ScraperArgs {
                    command: NtScraperCommands::Url { url: url.clone() },
                };
                let storage_guard = storage.read().await;
                handle_command(args, &*storage_guard).await?;
            }
        },
    }

    Ok(())
} 