pub mod scrapers;
pub mod cli;
#[macro_use]
mod logging;

pub use scrapers::ScraperManager;

pub use cli::{ScraperArgs, ScraperCommands, handle_command};
pub use scrapers::Scraper;

pub mod prelude {
    pub use super::scrapers::Scraper;
    pub use nt_core::{Article, Result, Error};
} 