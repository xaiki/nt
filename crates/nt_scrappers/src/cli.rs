use clap::{Parser, Subcommand};
use nt_core::{Result, ArticleStatus, Scraper};
use crate::{ScraperManager, };
use tracing::info;

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
            let articles = manager.scrape_source(source.as_deref()).await?;
            for article in articles {
                info!("📰 Article: {}", article.title);
                info!("   Source: {}", article.source);
                info!("   URL: {}", article.url);
                if let Some(summary) = &article.summary {
                    info!("   Summary: {}", summary);
                }
                if !article.related_articles.is_empty() {
                    info!("   Related Articles:");
                    for related in article.related_articles {
                        info!("     - {} ({}): {:.2}% similar", 
                            related.article.title,
                            related.article.source,
                            related.similarity_score.map(|score| score * 100.0).unwrap_or(0.0)
                        );
                    }
                }
                info!("");
            }
        }
        ScraperCommands::List => {
            manager.list_scrapers().await?;
        }
        ScraperCommands::Url { url } => {
            let article = manager.scrape_url(&url).await?;
            info!("📰 Article: {}", article.title);
            info!("   Source: {}", article.source);
            info!("   URL: {}", article.url);
            if let Some(summary) = &article.summary {
                info!("   Summary: {}", summary);
            }
            if !article.related_articles.is_empty() {
                info!("   Related Articles:");
                for related in article.related_articles {
                    info!("     - {} ({}): {:.2}% similar", 
                        related.article.title,
                        related.article.source,
                        related.similarity_score.map(|score| score * 100.0).unwrap_or(0.0)
                    );
                }
            }
            info!("");
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