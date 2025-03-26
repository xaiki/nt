use nt_core::{Article, ArticleSection, Result};
use super::InferenceModel;
use std::sync::Arc;
use std::fmt;

pub struct DivergenceAnalyzer {
    model: Arc<dyn InferenceModel>,
}

impl fmt::Debug for DivergenceAnalyzer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DivergenceAnalyzer")
            .field("model", &"<dyn InferenceModel>")
            .finish()
    }
}

impl DivergenceAnalyzer {
    pub fn new(model: Arc<dyn InferenceModel>) -> Self {
        Self { model }
    }

    pub async fn analyze_article(&self, article: &Article) -> Result<DivergenceAnalysis> {
        let mut analysis = DivergenceAnalysis {
            article_url: article.url.clone(),
            article_title: article.title.clone(),
            sections: Vec::new(),
        };

        for section in &article.sections {
            let section_analysis = self.analyze_section(section).await?;
            analysis.sections.push(section_analysis);
        }

        Ok(analysis)
    }

    async fn analyze_section(&self, section: &ArticleSection) -> Result<SectionAnalysis> {
        let embedding = self.model.generate_embeddings(&section.content).await?;
        
        Ok(SectionAnalysis {
            content: section.content.clone(),
            embedding: Some(embedding),
            divergence_score: None,
            similar_sections: Vec::new(),
        })
    }
}

#[derive(Debug)]
pub struct DivergenceAnalysis {
    pub article_url: String,
    pub article_title: String,
    pub sections: Vec<SectionAnalysis>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct SectionAnalysis {
    #[allow(dead_code)]
    content: String,
    embedding: Option<Vec<f32>>,
    divergence_score: Option<f32>,
    similar_sections: Vec<SimilarSection>,
}

#[derive(Debug)]
pub struct SimilarSection {
    pub content: String,
    pub similarity_score: f32,
    pub source_url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Divergence {
    pub source: String,
    pub section: String,
    pub summary: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DeepSeekModel;
    use chrono::Utc;

    #[tokio::test]
    async fn test_divergence_analyzer() {
        let model = Arc::new(DeepSeekModel::new(None).unwrap());
        let analyzer = DivergenceAnalyzer::new(model);

        let article = Article {
            url: "http://test.com".to_string(),
            title: "Test Article".to_string(),
            content: "This is a test article about politics.".to_string(),
            published_at: Utc::now(),
            source: "test".to_string(),
            sections: vec![],
            summary: None,
            authors: vec!["Test Author".to_string()],
        };

        let analysis = analyzer.analyze_article(&article).await.unwrap();
        assert_eq!(analysis.article_url, article.url);
    }
} 