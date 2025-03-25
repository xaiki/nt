pub mod cli;
pub mod scrapers;

pub use cli::{ScraperArgs, ScraperCommands, handle_command};
pub use scrapers::Scraper;

pub mod prelude {
    pub use super::scrapers::Scraper;
    pub use nt_core::{Article, Result, Error};
} 