use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use crate::scrapers::{Scraper, ScraperType};
use nt_core::RegionMetadata;
pub mod clarin;
pub mod lanacion;
pub mod lavoz;

pub use clarin::ClarinScraper;
pub use lanacion::LaNacionScraper;
pub use lavoz::LaVozScraper;

pub const REGION: RegionMetadata = RegionMetadata {
    name: "Argentina",
    emoji: "🇦🇷",
};

#[async_trait]
pub trait ArgentinaScraper: Scraper {}

#[async_trait]
impl ArgentinaScraper for ClarinScraper {}

#[async_trait]
impl ArgentinaScraper for LaNacionScraper {}

#[async_trait]
impl ArgentinaScraper for LaVozScraper {}

/// Returns a vector of all available Argentine newspaper scrapers
pub fn get_scrapers() -> Vec<Arc<Mutex<ScraperType>>> {
    vec![
        Arc::new(Mutex::new(ScraperType::Clarin(ClarinScraper::new()))),
        Arc::new(Mutex::new(ScraperType::LaNacion(LaNacionScraper::new()))),
        Arc::new(Mutex::new(ScraperType::LaVoz(LaVozScraper::new()))),
    ]
}

pub fn get_all_scrapers() -> Vec<Arc<Mutex<dyn Scraper>>> {
    vec![
        Arc::new(Mutex::new(ClarinScraper::new())),
        Arc::new(Mutex::new(LaNacionScraper::new())),
        Arc::new(Mutex::new(LaVozScraper::new())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_scrapers() {
        let scrapers = get_scrapers();
        assert_eq!(scrapers.len(), 3);
        
        let clarin = scrapers[0].lock().unwrap();
        assert!(clarin.can_handle("https://www.clarin.com/article"));
        assert!(!clarin.can_handle("https://www.lanacion.com.ar/article"));
        
        let lanacion = scrapers[1].lock().unwrap();
        assert!(lanacion.can_handle("https://www.lanacion.com.ar/article"));
        assert!(!lanacion.can_handle("https://www.clarin.com/article"));
        
        let lavoz = scrapers[2].lock().unwrap();
        assert!(lavoz.can_handle("https://www.lavoz.com.ar/article"));
        assert!(!lavoz.can_handle("https://www.clarin.com/article"));
    }

    #[tokio::test]
    async fn test_get_all_scrapers() {
        let scrapers = get_all_scrapers();
        assert_eq!(scrapers.len(), 3);
    }
} 