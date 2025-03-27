use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use clap::{Parser, Subcommand};
use nt_core::{Result, storage::ArticleStorage};
use crate::scrapers::{self, ArticleStatus, ScraperManager, ScraperType};
use crate::logging;
use tracing;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct ScraperArgs {
    #[command(subcommand)]
    pub command: ScraperCommands,
}

#[derive(Subcommand, Clone)]
pub enum ScraperCommands {
    /// Scrape articles from a specific source or all sources if none specified
    Source {
        /// The source to scrape in format country/source (e.g. argentina/clarin). If not specified, scrapes all sources.
        source: Option<String>,
    },
    /// List available scrapers
    List,
    /// Scrape a single URL
    Url {
        /// The URL to scrape
        url: String,
    },
}

pub async fn handle_command(args: ScraperArgs, storage: &dyn ArticleStorage) -> Result<()> {
    match args.command {
        ScraperCommands::Source { source } => {
            let mut manager = ScraperManager::new(storage);
            
            if source.is_none() || source.as_ref().unwrap().is_empty() {
                // If no source specified, scrape all regions and scrapers
                for (_, scrapers) in get_all_scrapers() {
                    for scraper in scrapers {
                        let s = scraper.lock().unwrap();
                        let cloned = s.clone();
                        drop(s); // Release the lock
                        manager.add_scraper(cloned);
                    }
                }
            } else {
                let (country, name) = parse_source(source.as_ref().unwrap())?;
                
                if let Some(name) = name {
                    // Scrape a specific source
                    let scraper = get_scraper(country, name)?;
                    manager.add_scraper(scraper);
                } else {
                    // Scrape all sources for the country
                    if let Some(scrapers) = get_all_scrapers().get(country) {
                        for scraper in scrapers {
                            let s = scraper.lock().unwrap();
                            let cloned = s.clone();
                            drop(s); // Release the lock
                            manager.add_scraper(cloned);
                        }
                    } else {
                        return Err(nt_core::Error::Scraping(format!(
                            "Country not supported: {}",
                            country
                        )));
                    }
                }
            }
            
            let results = manager.scrape_all().await?;
            for (article, status) in results {
                let emoji = match status {
                    ArticleStatus::New => "💥",
                    ArticleStatus::Updated => "👻",
                    ArticleStatus::Unchanged => "✅",
                };
                let authors = if article.authors.is_empty() {
                    "".to_string()
                } else {
                    format!(" | by {}", article.authors.join(", "))
                };
                println!("{} {} - {}{}", emoji, article.title, article.url, authors);
            }
        }
        ScraperCommands::List => {
            println!("Available scrapers:");
            for (country, scrapers) in get_all_scrapers() {
                println!("{}:", country);
                for scraper in scrapers {
                    let scraper = scraper.lock().unwrap();
                    let aliases = scraper.cli_names();
                    let alias_str = if !aliases.is_empty() {
                        format!(" (aliases: {})", aliases.join(", "))
                    } else {
                        String::new()
                    };
                    println!("  {}/{} - {}{}", 
                        country,
                        aliases.first().unwrap_or(&scraper.source_metadata().name.to_lowercase().as_str()),
                        scraper.source_metadata().name,
                        alias_str
                    );
                }
            }
        }
        ScraperCommands::Url { url } => {
            let mut manager = ScraperManager::new(storage);
            
            // Add all available scrapers
            for (_, scrapers) in get_all_scrapers() {
                for scraper in scrapers {
                    let s = scraper.lock().unwrap();
                    let cloned = s.clone();
                    drop(s); // Release the lock
                    manager.add_scraper(cloned);
                }
            }
            
            // Try to scrape the URL
            match manager.scrape_url(&url).await {
                Ok((article, status)) => {
                    let emoji = match status {
                        ArticleStatus::New => "💥",
                        ArticleStatus::Updated => "👻",
                        ArticleStatus::Unchanged => "✅",
                    };
                    let authors = if article.authors.is_empty() {
                        "".to_string()
                    } else {
                        format!(" | by \x1b[1m{}\x1b[0m", article.authors.join(", "))
                    };
                    println!("{} {} - {}{}", emoji, article.title, article.url, authors);
                }
                Err(e) => eprintln!("Failed to scrape {}: {}", url, e),
            }
        }
    }
    Ok(())
}

fn parse_source(source: &str) -> Result<(&str, Option<&str>)> {
    let parts: Vec<&str> = source.split('/').collect();
    if parts.len() > 2 {
        return Err(nt_core::Error::Scraping(
            "Invalid source format. Expected: country or country/source".to_string(),
        ));
    }
    Ok((parts[0], parts.get(1).copied()))
}

fn get_all_scrapers() -> HashMap<String, Vec<Arc<Mutex<ScraperType>>>> {
    let mut scrapers = HashMap::new();
    
    // Add scrapers for each country
    scrapers.insert("argentina".to_string(), scrapers::argentina::get_scrapers());
    // Add more countries here as they are implemented
    
    scrapers
}

fn get_scraper(country: &str, name: &str) -> Result<ScraperType> {
    let all_scrapers = get_all_scrapers();
    
    if let Some(scrapers) = all_scrapers.get(country) {
        // Try to find a scraper that matches either by name or CLI names
        for scraper in scrapers {
            let s = scraper.lock().unwrap();
            if s.source_metadata().name.to_lowercase().replace('í', "i") == name.to_lowercase().replace('í', "i") 
               || s.cli_names().contains(&name) {
                let cloned = s.clone();
                drop(s); // Release the lock
                return Ok(cloned);
            }
        }
        
        Err(nt_core::Error::Scraping(format!("Scraper not found: {}/{}", country, name)))
    } else {
        Err(nt_core::Error::Scraping(format!(
            "Country not supported: {}",
            country
        )))
    }
}

pub async fn list_scrapers(storage: &dyn ArticleStorage) -> Result<()> {
    let manager = ScraperManager::new(storage);
    let mut logger = logging::init_logging();
    tracing::info!("Available scrapers:");
    for scraper in manager.get_scrapers() {
        let metadata = scraper.source_metadata();
        let prefix = format!("{} {} {}", metadata.region.emoji, metadata.emoji, metadata.name);
        logger = logger.with_prefix(prefix);
        logger.info(&metadata.region.name);
    }
    Ok(())
}

pub async fn scrape_url(url: &str, storage: &dyn ArticleStorage) -> Result<()> {
    let mut manager = ScraperManager::new(storage);
    let scraper = manager.get_scraper_for_url(url)?;
    let metadata = scraper.source_metadata();
    let prefix = format!("{} {} {}", metadata.region.emoji, metadata.emoji, metadata.name);
    let logger = logging::init_logging().with_prefix(prefix);
    logger.info(&format!("Scraping article from {}", url));
    let article = manager.scrape_url(url).await?.0;
    logger.info(&format!("Successfully scraped article: {}", article.title));
    Ok(())
}

pub async fn scrape_source(source: Option<&str>, storage: &dyn ArticleStorage) -> Result<()> {
    let manager = ScraperManager::new(storage);
    let scrapers = if let Some(source) = source {
        manager.get_scrapers_for_source(source)?
    } else {
        manager.get_scrapers().iter().collect()
    };

    let mut logger = logging::init_logging();

    for scraper in scrapers {
        let metadata = scraper.source_metadata();
        let prefix = format!("{} {} {}", metadata.region.emoji, metadata.emoji, metadata.name);
        logger = logger.with_prefix(prefix);
        logger.info("Starting scrape");
        
        let mut manager = ScraperManager::new(storage);
        manager.add_scraper(scraper.clone());
        if let Err(e) = manager.scrape_all().await {
            logger.error(&format!("Failed to scrape: {}", e));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_scraper() {
        let scraper = get_scraper("argentina", "clarín");
        assert!(scraper.is_ok());

        let scraper = get_scraper("argentina", "invalid");
        assert!(scraper.is_err());

        let scraper = get_scraper("invalid", "scraper");
        assert!(scraper.is_err());

        let scraper = get_scraper("invalid", "");
        assert!(scraper.is_err());
    }
} 