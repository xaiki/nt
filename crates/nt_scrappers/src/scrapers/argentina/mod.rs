use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use crate::scrapers::Scraper;

pub mod clarin;
pub mod lanacion;
pub mod lavoz;

pub use clarin::ClarinScraper;
pub use lanacion::LaNacionScraper;
pub use lavoz::LaVozScraper;

#[async_trait]
pub trait ArgentinaScraper: Scraper {}

#[async_trait]
impl ArgentinaScraper for ClarinScraper {}

#[async_trait]
impl ArgentinaScraper for LaNacionScraper {}

#[async_trait]
impl ArgentinaScraper for LaVozScraper {}

/// Returns a vector of all available Argentine newspaper scrapers
pub fn get_scrapers() -> Vec<Arc<Mutex<dyn Scraper>>> {
    vec![
        Arc::new(Mutex::new(ClarinScraper::new())),
        Arc::new(Mutex::new(LaNacionScraper::new())),
        Arc::new(Mutex::new(LaVozScraper::new())),
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
        assert!(!scrapers.is_empty());
        
        // Test that each scraper can handle its own URLs
        let clarin_url = "https://www.clarin.com/some-article";
        let lanacion_url = "https://www.lanacion.com.ar/some-article";
        let lavoz_url = "https://www.lavoz.com.ar/some-article";

        assert!(scrapers.iter().any(|s| s.lock().unwrap().can_handle(clarin_url)));
        assert!(scrapers.iter().any(|s| s.lock().unwrap().can_handle(lanacion_url)));
        assert!(scrapers.iter().any(|s| s.lock().unwrap().can_handle(lavoz_url)));
    }

    #[tokio::test]
    async fn test_get_all_scrapers() {
        let scrapers = get_all_scrapers();
        assert_eq!(scrapers.len(), 3);
    }
} 