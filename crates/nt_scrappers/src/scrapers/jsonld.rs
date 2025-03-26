use scraper::{Html, Selector};
use serde_json;

/// Extracts authors from JSON-LD metadata in the HTML document.
/// Returns a vector of author names.
pub fn extract_authors(document: &Html) -> Vec<String> {
    let mut authors = Vec::new();

    if let Ok(script_selector) = Selector::parse("script[type='application/ld+json']") {
        for script in document.select(&script_selector) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(script.text().collect::<String>().trim()) {
                // Try to get author from the JSON-LD data
                if let Some(author) = json.get("author") {
                    match author {
                        serde_json::Value::Array(arr) => {
                            for author_obj in arr {
                                if let Some(name) = author_obj.get("name").and_then(|n| n.as_str()) {
                                    authors.push(name.trim().to_string());
                                }
                            }
                        }
                        serde_json::Value::Object(obj) => {
                            if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                                authors.push(name.trim().to_string());
                            }
                        }
                        serde_json::Value::String(s) => {
                            authors.push(s.trim().to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    authors
} 