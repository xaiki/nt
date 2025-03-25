use clap::{Parser, Subcommand};
use nt_core::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scraper-related commands
    Scrapers(nt_scrappers::ScraperArgs),
    // Add other crate commands here as they become available
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scrapers(args) => {
            nt_scrappers::handle_command(args).await?;
        }
    }

    Ok(())
} 