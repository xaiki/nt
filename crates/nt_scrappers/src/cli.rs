use clap::{Parser, Subcommand};
use nt_core::{Result, ArticleStatus, Scraper};
use crate::{ScraperManager, };

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

pub async fn handle_command(args: ScraperArgs, manager: &mut ScraperManager) -> Result<()> {
    match args.command {
        ScraperCommands::Source { source } => {
            if source.is_none() || source.as_ref().unwrap().is_empty() {
                // If no source specified, scrape all regions and scrapers
                for scraper in crate::scrapers::argentina::get_scrapers() {
                    let s = scraper.lock().unwrap();
                    let cloned = s.clone();
                    drop(s); // Release the lock
                    manager.add_scraper(cloned);
                }
            } else {
                let scrapers = manager.get_scrapers_for_source(source.as_ref().unwrap())?;
                for scraper in scrapers {
                    manager.add_scraper(scraper);
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
            for scraper in manager.get_scrapers() {
                let s = scraper.lock().unwrap();
                let aliases = s.cli_names();
                let alias_str = if !aliases.is_empty() {
                    format!(" (aliases: {})", aliases.join(", "))
                } else {
                    String::new()
                };
                println!("  {}/{} - {}{}", 
                    "argentina",
                    aliases.first().unwrap_or(&s.source_metadata().name.to_lowercase().as_str()),
                    s.source_metadata().name,
                    alias_str
                );
            }
        }
        ScraperCommands::Url { url } => {
            // Add all available scrapers
            for scraper in crate::scrapers::argentina::get_scrapers() {
                let s = scraper.lock().unwrap();
                let cloned = s.clone();
                drop(s); // Release the lock
                manager.add_scraper(cloned);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_command() {
        // TODO: Add tests for handle_command
    }
} 