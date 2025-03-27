use std::fmt;
use nt_core::{Result, Article, ArticleSection};
use super::InferenceModel;

pub struct DeepSeekModel {
    api_key: Option<String>,
}

impl fmt::Debug for DeepSeekModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeepSeekModel")
            .field("api_key", &self.api_key.as_deref().map(|_| "<redacted>"))
            .finish()
    }
}

impl DeepSeekModel {
    pub fn new(api_key: Option<String>) -> Result<Self> {
        if api_key.is_none() {
            return Err(nt_core::Error::Inference("DeepSeek API key is required".to_string()));
        }
        Ok(Self { api_key })
    }
}

#[async_trait::async_trait]
impl InferenceModel for DeepSeekModel {
    fn name(&self) -> &str {
        "DeepSeek"
    }

    async fn summarize_article(&self, article: &Article) -> Result<String> {
        if self.api_key.is_none() {
            return Err(nt_core::Error::Inference("DeepSeek API key is required".to_string()));
        }
        // For now, just return the first few sentences of the content
        let sentences: Vec<&str> = article.content
            .split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .take(3)
            .collect();
        
        let summary = sentences.join(". ") + ".";
        tracing::debug!("Generated summary from content: {}", summary);
        Ok(summary)
    }

    async fn summarize_sections(&self, sections: &[ArticleSection]) -> Result<Vec<String>> {
        if self.api_key.is_none() {
            return Err(nt_core::Error::Inference("DeepSeek API key is required".to_string()));
        }
        // For now, just return the first few sentences of each section
        Ok(sections.iter().map(|section| {
            let sentences: Vec<&str> = section.content
                .split(|c| c == '.' || c == '!' || c == '?')
                .filter(|s| !s.trim().is_empty())
                .take(2)
                .collect();
            
            let summary = sentences.join(". ") + ".";
            tracing::debug!("Generated section summary: {}", summary);
            summary
        }).collect())
    }

    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        if self.api_key.is_none() {
            return Err(nt_core::Error::Inference("DeepSeek API key is required".to_string()));
        }
        // For now, generate a simple embedding based on text length and character frequencies
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

    #[test]
    fn test_model_requires_api_key() {
        // Test that creating a model without an API key fails
        let result = DeepSeekModel::new(None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "DeepSeek API key is required");

        // Test that creating a model with an API key succeeds
        let result = DeepSeekModel::new(Some("test-key".to_string()));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_model_methods_require_api_key() {
        let model = DeepSeekModel::new(Some("test-key".to_string()));
        assert!(model.is_ok());
        let model = model.unwrap();

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