use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use clap::{Parser, Subcommand};
use nt_core::{Result, storage::ArticleStorage};
use crate::scrapers::{self, ArticleStatus, ScraperManager, ScraperType};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct ScraperArgs {
    #[command(subcommand)]
    pub command: ScraperCommands,
}

#[derive(Subcommand)]
pub enum ScraperCommands {
    /// Scrape articles from a specific source
    Scrape {
        /// The source to scrape in format country/source (e.g. argentina/clarin)
        source: String,
    },
    /// List available scrapers
    List,
}

pub async fn handle_command(args: ScraperArgs, storage: &dyn ArticleStorage) -> Result<()> {
    match args.command {
        ScraperCommands::Scrape { source } => {
            let (country, name) = parse_source(&source)?;
            let scraper = get_scraper(country, name)?;
            
            let mut manager = ScraperManager::new(storage);
            manager.add_scraper(scraper);
            
            let results = manager.scrape_all().await?;
            for (article, status) in results {
                let emoji = match status {
                    ArticleStatus::New => "💥",
                    ArticleStatus::Updated => "📝",
                    ArticleStatus::Unchanged => "⏭️",
                };
                println!("{} {} - {}", emoji, article.title, article.url);
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
                        aliases.first().unwrap_or(&scraper.source().to_lowercase().as_str()),
                        scraper.source(),
                        alias_str
                    );
                }
            }
        }
    }
    Ok(())
}

fn parse_source(source: &str) -> Result<(&str, &str)> {
    let parts: Vec<&str> = source.split('/').collect();
    if parts.len() != 2 {
        return Err(nt_core::Error::Scraping(
            "Invalid source format. Expected: country/source".to_string(),
        ));
    }
    Ok((parts[0], parts[1]))
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
            if s.source().to_lowercase().replace('í', "i") == name.to_lowercase().replace('í', "i") 
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