use std::fmt;
use nt_core::{Result, Article, ArticleSection};
use super::{InferenceModel, Config};

pub struct DummyModel;

impl fmt::Debug for DummyModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DummyModel").finish()
    }
}

impl DummyModel {
    pub async fn new(_config: Option<Config>) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait::async_trait]
impl InferenceModel for DummyModel {
    fn name(&self) -> &str {
        "Dummy"
    }

    async fn summarize_article(&self, article: &Article) -> Result<String> {
        // Take first 20 words and join them
        let words: Vec<&str> = article.content.split_whitespace().take(20).collect();
        Ok(words.join(" "))
    }

    async fn summarize_sections(&self, sections: &[ArticleSection]) -> Result<Vec<String>> {
        // For each section, take first 20 words
        Ok(sections.iter().map(|section| {
            let words: Vec<&str> = section.content.split_whitespace().take(20).collect();
            words.join(" ")
        }).collect())
    }

    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        // Generate a simple embedding based on text length and character frequencies
        let mut embedding = vec![0.0; 768];
        
        // Use text length as a feature
        let text_len = text.len() as f32;
        embedding[0] = text_len / 1000.0; // Normalize to roughly [0,1] range
        
        // Use character frequencies as features
        let mut char_freq = std::collections::HashMap::new();
        for c in text.chars() {
            *char_freq.entry(c).or_insert(0) += 1;
        }
        
        // Fill the embedding with character frequencies
        for (i, (_, &count)) in char_freq.iter().enumerate().take(767) {
            embedding[i + 1] = count as f32 / text_len;
        }
        
        Ok(embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nt_core::Article;
    use chrono::Utc;

    #[tokio::test]
    async fn test_dummy_model() {
        let model = DummyModel::new(None).await.unwrap();

        // Test article summarization
        let article = Article {
            url: "http://test.com".to_string(),
            title: "Test Article".to_string(),
            content: "This is a test article. It has multiple sentences. This is the third sentence.".to_string(),
            published_at: Utc::now(),
            source: "test".to_string(),
            sections: vec![],
            summary: None,
            authors: vec!["Test Author".to_string()],
            related_articles: Vec::new(),
        };

        let result = model.summarize_article(&article).await;
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert!(!summary.is_empty());
        assert!(summary.contains("This is a test article"));

        // Test section summarization
        let section = nt_core::ArticleSection {
            content: "This is a test section. It has multiple sentences.".to_string(),
            summary: None,
            embedding: None,
        };
        let result = model.summarize_sections(&[section]).await;
        assert!(result.is_ok());
        let summaries = result.unwrap();
        assert!(!summaries.is_empty());
        assert!(summaries[0].contains("This is a test section"));

        // Test embedding generation
        let result = model.generate_embeddings("Test text").await;
        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 768);
        assert!(embedding[0] > 0.0); // Text length feature should be non-zero
    }
} 