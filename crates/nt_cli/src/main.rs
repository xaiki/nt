use clap::Parser;
use nt_core::{Result, storage::ArticleStorage, Article};
use nt_storage::{InMemoryStorage, StorageBackend};
use chrono::Utc;
use nt_scrappers::cli::{ScraperArgs, ScraperCommands as NtScraperCommands, handle_command};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[cfg(feature = "chroma")]
use nt_storage::ChromaDBStorage;

#[cfg(feature = "qdrant")]
use nt_storage::QdrantStorage;

#[cfg(feature = "sqlite")]
use nt_storage::SQLiteStorage;

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
        command: ScraperCommands,
    },
}

#[derive(clap::Subcommand, Debug)]
enum ScraperCommands {
    Source {
        source: String,
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

    // Check storage health before proceeding
    if let Err(e) = check_storage(&storage).await {
        eprintln!("Storage health check failed: {}", e);
        return Err(e);
    }

    match cli.command {
        Commands::Scrape { command } => match command {
            ScraperCommands::Source { source } => {
                info!("Scraping articles from {}", source);
                let args = ScraperArgs {
                    command: NtScraperCommands::Source { source: source.clone() },
                };
                let storage_guard = storage.read().await;
                handle_command(args, &*storage_guard).await?;
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