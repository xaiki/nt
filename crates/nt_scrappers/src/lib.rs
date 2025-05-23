pub mod manager;
pub mod cli;
pub mod scrapers;
pub mod logging;

pub use scrapers::ScraperType;
pub use nt_core::{Scraper, ArticleStatus, SourceMetadata, RegionMetadata};
pub use cli::{ScraperArgs, ScraperCommands, handle_command};
pub use manager::ScraperManager;

pub mod prelude {
    pub use nt_core::{Article, Result, Error, Scraper};
} 