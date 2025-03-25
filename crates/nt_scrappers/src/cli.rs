use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use clap::{Args, Subcommand};
use nt_core::Result;
use crate::scrapers::{self, Scraper, ArticleStatus};

#[derive(Args)]
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

pub async fn handle_command(args: ScraperArgs) -> Result<()> {
    match args.command {
        ScraperCommands::Scrape { source } => {
            let (country, name) = parse_source(&source)?;
            let scraper = get_scraper(country, name)?;
            
            let urls = scraper.lock().unwrap().get_article_urls().await?;
            println!("Found {} articles", urls.len());
            
            for url in urls {
                let mut scraper = scraper.lock().unwrap();
                match scraper.scrape_article(&url).await {
                    Ok((article, status)) => {
                        let emoji = match status {
                            ArticleStatus::New => "🆕",
                            ArticleStatus::Updated => "📝",
                            ArticleStatus::Unchanged => "⏭️",
                        };
                        println!("{} {} - {}", emoji, article.title, url);
                    }
                    Err(e) => {
                        eprintln!("Failed to scrape {}: {}", url, e);
                    }
                }
            }
        }
        ScraperCommands::List => {
            // TODO: Implement listing available scrapers
            println!("Available scrapers:");
            println!("  argentina/clarin");
            println!("  argentina/lanacion");
            println!("  argentina/lavoz");
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

fn get_scraper(country: &str, name: &str) -> Result<Arc<Mutex<dyn Scraper>>> {
    match country {
        "argentina" => {
            let scrapers = scrapers::argentina::get_scrapers();
            scrapers
                .into_iter()
                .find(|s| s.lock().unwrap().source().to_lowercase().replace('í', "i") == name.to_lowercase().replace('í', "i"))
                .ok_or_else(|| {
                    nt_core::Error::Scraping(format!("Scraper not found: {}/{}", country, name))
                })
        }
        _ => Err(nt_core::Error::Scraping(format!(
            "Country not supported: {}",
            country
        ))),
    }
}

pub async fn scrape_articles(scrapers: &[Arc<Mutex<dyn Scraper>>]) -> Result<()> {
    let mut article_cache = HashMap::new();

    for scraper in scrapers {
        let urls = scraper.lock().unwrap().get_article_urls().await?;
        for url in urls {
            if let Some(scraper) = scraper.lock().unwrap().can_handle(&url).then(|| scraper.clone()) {
                let mut scraper = scraper.lock().unwrap();
                match scraper.scrape_article(&url).await {
                    Ok((article, status)) => {
                        let emoji = match status {
                            ArticleStatus::New => "🆕",
                            ArticleStatus::Updated => "📝",
                            ArticleStatus::Unchanged => "⏭️",
                        };
                        println!("{} {} - {}", emoji, article.title, url);
                        article_cache.insert(url, article);
                    }
                    Err(e) => eprintln!("Error scraping {}: {}", url, e),
                }
            }
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