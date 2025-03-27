use std::sync::Arc;
use std::fmt;
use nt_core::{Result, Article, ArticleSection};
use super::{InferenceModel, Config};
use crate::ModelConfig;
use crate::InferenceConfig;
use nt_storage::{BackendConfig, EmbeddingModel};
use anyhow::anyhow;
use url::Url;

#[cfg(feature = "qdrant")]
use nt_storage::backends::qdrant::QdrantConfig;

#[cfg(feature = "ollama")]
use {
    langchain_rust::llm::ollama::client::{Ollama, OllamaClient},
    langchain_rust::llm::client::GenerationOptions,
    langchain_rust::language_models::llm::LLM,
};

#[derive(Debug)]
pub struct LangChainModelConfig {
    ollama_host: String,
    ollama_port: u16,
    model_name: String,
}

impl Default for LangChainModelConfig {
    fn default() -> Self {
        Self {
            ollama_host: "http://localhost".to_string(),
            ollama_port: 11434,
            model_name: "gemma3:12b".to_string(),
        }
    }
}

impl ModelConfig for LangChainModelConfig {
    fn from_inference_config(config: &InferenceConfig) -> Self {
        let url = config.model_url.clone().unwrap_or_else(|| "http://localhost:11434/gemma3:12b".to_string());
        let parsed_url = Url::parse(&url).unwrap_or_else(|_| Url::parse("http://localhost:11434/gemma3:12b").unwrap());
        
        // Extract model name from path, defaulting to gemma3:12b if not specified
        let model_name = parsed_url.path()
            .trim_start_matches('/')
            .to_string();

        Self {
            ollama_host: parsed_url.scheme().to_string() + "://" + parsed_url.host_str().unwrap_or("localhost"),
            ollama_port: parsed_url.port().unwrap_or(11434),
            model_name: if model_name.is_empty() { "gemma3:12b".to_string() } else { model_name },
        }
    }
}

impl LangChainModelConfig {
    pub fn get_ollama_host(&self) -> &str {
        &self.ollama_host
    }

    pub fn get_ollama_port(&self) -> u16 {
        self.ollama_port
    }

    pub fn get_model_name(&self) -> &str {
        &self.model_name
    }
}

pub struct LangChainModel {
    #[cfg(feature = "ollama")]
    ollama_client: Option<Ollama>,
}

impl fmt::Debug for LangChainModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LangChainModel")
            .field("ollama_client", &"<Ollama>")
            .finish()
    }
}

impl LangChainModel {
    pub async fn new(config: Option<Config>) -> Result<Self> {
        #[cfg(feature = "ollama")]
        let ollama_client = if let Some(config) = config {
            let model_config = LangChainModelConfig::from_inference_config(&config.inference_config);
            let client = Arc::new(OllamaClient::new(
                model_config.get_ollama_host(),
                model_config.get_ollama_port(),
            ));
            
            // Check if Ollama is available by making a test request
            let test_ollama = Ollama::new(
                client.clone(),
                model_config.get_model_name().to_string(),
                Some(GenerationOptions::default()),
            );
            
            // Try to make a test request
            if let Err(e) = test_ollama.invoke("test").await {
                return Err(nt_core::Error::External(anyhow!(
                    "Ollama is not available at {}:{}: {}. Please ensure Ollama is running and the model '{}' is installed.",
                    model_config.get_ollama_host(),
                    model_config.get_ollama_port(),
                    e,
                    model_config.get_model_name()
                )));
            }
            
            Some(test_ollama)
        } else {
            None
        };

        Ok(Self {
            #[cfg(feature = "ollama")]
            ollama_client,
        })
    }
}

#[async_trait::async_trait]
impl InferenceModel for LangChainModel {
    fn name(&self) -> &str {
        "LangChain"
    }

    async fn summarize_article(&self, article: &Article) -> Result<String> {
        #[cfg(feature = "ollama")]
        {
            if let Some(ollama) = &self.ollama_client {
                let prompt = format!("Please summarize the following article:\n\n{}", article.content);
                let response = ollama.invoke(&prompt)
                    .await
                    .map_err(|e| nt_core::Error::External(anyhow!("Failed to generate summary: {}", e)))?;
                return Ok(response);
            }
        }
        // Fallback to basic summary if Ollama is not available
        Ok(format!("Summary of: {}", article.title))
    }

    async fn summarize_sections(&self, sections: &[ArticleSection]) -> Result<Vec<String>> {
        #[cfg(feature = "ollama")]
        {
            if let Some(ollama) = &self.ollama_client {
                let mut summaries = Vec::new();
                for section in sections {
                    let prompt = format!("Please summarize the following section:\n\n{}", section.content);
                    let response = ollama.invoke(&prompt)
                        .await
                        .map_err(|e| nt_core::Error::External(anyhow!("Failed to generate section summary: {}", e)))?;
                    summaries.push(response);
                }
                return Ok(summaries);
            }
        }
        // Fallback to basic summaries if Ollama is not available
        Ok(sections.iter().map(|s| format!("Summary of section: {}", s.content)).collect())
    }

    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>> {
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