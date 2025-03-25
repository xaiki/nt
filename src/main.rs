mod error;
mod models;
mod scraper;

use scraper::kiosko::KioskoScraper;
use scraper::NewsScraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scraper = KioskoScraper::new();
    
    println!("Fetching articles from {}", scraper.source_name());
    let articles = scraper.fetch_articles().await?;
    
    println!("Found {} articles", articles.len());
    for article in articles {
        println!("- {}", article.title);
    }
    
    Ok(())
}
