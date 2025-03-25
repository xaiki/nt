use std::sync::Arc;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use nt_core::{Article, ArticleSection, Result};
use std::fmt;

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

pub struct DeepSeekModel {
    client: Arc<Client>,
    api_key: String,
    base_url: String,
}

impl DeepSeekModel {
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let client = Arc::new(Client::new());
        Ok(Self {
            client,
            api_key: api_key.unwrap_or_default(),
            base_url: "https://api.deepseek.com/v1".to_string(),
        })
    }
}

impl fmt::Debug for DeepSeekModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeepSeekModel")
            .field("client", &"<reqwest::Client>")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[async_trait]
impl super::InferenceModel for DeepSeekModel {
    fn name(&self) -> &str {
        "DeepSeek"
    }

    async fn summarize_article(&self, article: &Article) -> Result<String> {
        let prompt = format!(
            "Please summarize the following article:\n\nTitle: {}\n\nContent: {}\n\nSummary:",
            article.title, article.content
        );
        
        let request = ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;

        Ok(response.choices[0].message.content.clone())
    }

    async fn summarize_sections(&self, sections: &[ArticleSection]) -> Result<Vec<String>> {
        let mut summaries = Vec::with_capacity(sections.len());
        for section in sections {
            let prompt = format!(
                "Please summarize the following section:\n\n{}\n\nSummary:",
                section.content
            );

            let request = ChatRequest {
                model: "deepseek-chat".to_string(),
                messages: vec![ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                }],
            };

            let response = self.client
                .post(format!("{}/chat/completions", self.base_url))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&request)
                .send()
                .await?
                .json::<ChatResponse>()
                .await?;

            summaries.push(response.choices[0].message.content.clone());
        }
        Ok(summaries)
    }

    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            input: text.to_string(),
            model: "deepseek-embedding".to_string(),
        };

        let response = self.client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<EmbeddingResponse>()
            .await?;

        Ok(response.data[0].embedding.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::InferenceModel;

    #[tokio::test]
    async fn test_deepseek_model() {
        let model = DeepSeekModel::new(None).unwrap();
        let article = Article {
            url: "test".to_string(),
            title: "Test Article".to_string(),
            content: "This is a test article content.".to_string(),
            summary: None,
            published_at: Utc::now(),
            source: "test".to_string(),
            sections: vec![
                ArticleSection {
                    content: "Section 1 content".to_string(),
                    summary: None,
                    embedding: None,
                }
            ],
        };

        let summary = model.summarize_article(&article).await;
        assert!(summary.is_ok());

        let section_summaries = model.summarize_sections(&article.sections).await;
        assert!(section_summaries.is_ok());

        let embeddings = model.generate_embeddings(&article.content).await;
        assert!(embeddings.is_ok());
    }
}

#[async_trait::async_trait]
pub trait InferenceModel: Send + Sync + fmt::Debug {
    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>>;
} 