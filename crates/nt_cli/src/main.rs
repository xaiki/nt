use clap::{Parser, Subcommand};
use nt_core::{Result, storage::ArticleStorage, Article};
use nt_storage::{InMemoryStorage, ChromaDBStorage, QdrantStorage, StorageBackend};
use chrono::Utc;

async fn check_storage(storage: &Box<dyn ArticleStorage>) -> Result<()> {
    let test_article = Article {
        url: "http://test.com".to_string(),
        title: "Test Article".to_string(),
        content: "Test content".to_string(),
        published_at: Utc::now(),
        source: "test".to_string(),
        sections: vec![],
        summary: None,
    };

    // Try to store the article
    storage.store_article(&test_article).await?;
    
    // Try to retrieve it
    let articles = storage.get_by_source("test").await?;
    if !articles.iter().any(|a| a.url == test_article.url) {
        return Err(nt_core::Error::Storage("Failed to retrieve test article".to_string()));
    }

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Storage backend to use (memory, chroma, or qdrant)
    #[arg(short, long, default_value = "memory")]
    storage: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Scraper-related commands
    Scrapers(nt_scrappers::ScraperArgs),
    // Add other crate commands here as they become available
}

async fn create_storage<T: StorageBackend + 'static>() -> Result<(Box<dyn ArticleStorage>, &'static str)> {
    match T::new().await {
        Ok(storage) => Ok((Box::new(storage) as Box<dyn ArticleStorage>, T::get_error_message())),
        Err(e) => {
            eprintln!("Failed to connect to {}: {}", std::any::type_name::<T>(), e);
            eprintln!("Please ensure: {}", T::get_error_message());
            Err(e)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let (storage, error_message) = match cli.storage.as_str() {
        "memory" => create_storage::<InMemoryStorage>().await?,
        "chroma" => create_storage::<ChromaDBStorage>().await?,
        "qdrant" => create_storage::<QdrantStorage>().await?,
        _ => return Err(nt_core::Error::Storage(format!("Unknown storage backend: {}", cli.storage))),
    };

    // Check storage health before proceeding
    if let Err(e) = check_storage(&storage).await {
        eprintln!("Storage health check failed: {}", e);
        eprintln!("Please ensure: {}", error_message);
        return Err(e);
    }

    match cli.command {
        Commands::Scrapers(args) => {
            nt_scrappers::handle_command(args, storage).await?;
        }
    }

    Ok(())
} 