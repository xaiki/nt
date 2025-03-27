use std::sync::Arc;
use std::fmt;
use nt_core::{Result, Article, ArticleSection};
use super::{InferenceModel, Config};
use crate::ModelConfig;
use crate::InferenceConfig;
use nt_storage::{BackendConfig, EmbeddingModel};
#[cfg(feature = "qdrant")]
use nt_storage::backends::qdrant::QdrantConfig;

#[cfg(feature = "qdrant")]
use langchain_rust::{
    embedding::{openai::{OpenAiEmbedder, OpenAIConfig}, Embedder},
    vectorstore::qdrant::{Qdrant, StoreBuilder, Store},
};

#[cfg(feature = "ollama")]
use langchain_rust::{
    llm::ollama::client::{Ollama, OllamaClient},
    llm::client::GenerationOptions,
    language_models::llm::LLM,
};

#[derive(Debug)]
pub struct LangChainModelConfig {
    ollama_host: String,
}

impl Default for LangChainModelConfig {
    fn default() -> Self {
        Self {
            ollama_host: "http://localhost:11434".to_string(),
        }
    }
}

impl ModelConfig for LangChainModelConfig {
    fn from_inference_config(config: &InferenceConfig) -> Self {
        Self {
            ollama_host: config.model_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string()),
        }
    }
}

impl LangChainModelConfig {
    pub fn get_ollama_host(&self) -> &str {
        &self.ollama_host
    }
}

pub struct LangChainModel {
    embedder: Arc<OpenAiEmbedder<OpenAIConfig>>,
    #[cfg(feature = "qdrant")]
    qdrant_store: Option<Arc<Store>>,
    #[cfg(feature = "ollama")]
    ollama_client: Option<Arc<Ollama>>,
    model_config: LangChainModelConfig,
}

impl LangChainModel {
    pub fn new(config: Option<Config>) -> Result<Self> {
        let openai_config = OpenAIConfig::default();
        let embedder = Arc::new(OpenAiEmbedder::new(openai_config));
        
        let model_config = if let Some(config) = &config {
            LangChainModelConfig::from_inference_config(&config.inference_config)
        } else {
            LangChainModelConfig::default()
        };

        #[cfg(feature = "qdrant")]
        let qdrant_store = None;

        #[cfg(feature = "ollama")]
        let ollama_client = Some(Arc::new(Ollama::new(
            Arc::new(OllamaClient::new(model_config.get_ollama_host(), 11434)),
            "llama2".to_string(),
            Some(GenerationOptions::default()),
        )));

        Ok(Self {
            embedder,
            #[cfg(feature = "qdrant")]
            qdrant_store,
            #[cfg(feature = "ollama")]
            ollama_client,
            model_config,
        })
    }

    pub async fn with_backend_config(mut self, config: &BackendConfig) -> Result<Self> {
        match config.embedding_model {
            EmbeddingModel::OpenAI => {
                // Already using OpenAI embeddings
                Ok(self)
            }
            EmbeddingModel::DeepSeek => {
                // Use DeepSeek embeddings
                let openai_config = OpenAIConfig::default();
                self.embedder = Arc::new(OpenAiEmbedder::new(openai_config));
                Ok(self)
            }
            #[cfg(feature = "qdrant")]
            EmbeddingModel::Qdrant => {
                let qdrant_config = QdrantConfig::new();
                let client = Qdrant::from_url(&qdrant_config.config.url)
                    .build()
                    .map_err(|e| nt_core::Error::External(anyhow::anyhow!("Failed to build Qdrant client: {}", e)))?;

                let openai_config = OpenAIConfig::default();
                let embedder = OpenAiEmbedder::new(openai_config);
                let store = StoreBuilder::new()
                    .embedder(embedder)
                    .client(client)
                    .collection_name(&qdrant_config.config.collection)
                    .build()
                    .await
                    .map_err(|e| nt_core::Error::External(anyhow::anyhow!("Failed to build Qdrant store: {}", e)))?;

                self.qdrant_store = Some(Arc::new(store));
                Ok(self)
            }
            #[cfg(not(feature = "qdrant"))]
            EmbeddingModel::Qdrant => {
                Err(nt_core::Error::External(anyhow::anyhow!("Qdrant feature not enabled")))
            }
            EmbeddingModel::SQLite => {
                // For SQLite, we'll use OpenAI embeddings since langchain_rust doesn't support SQLite embeddings
                let openai_config = OpenAIConfig::default();
                self.embedder = Arc::new(OpenAiEmbedder::new(openai_config));
                Ok(self)
            }
        }
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
                    .map_err(|e| nt_core::Error::External(anyhow::anyhow!("Failed to generate summary: {}", e)))?;
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
                        .map_err(|e| nt_core::Error::External(anyhow::anyhow!("Failed to generate section summary: {}", e)))?;
                    summaries.push(response);
                }
                return Ok(summaries);
            }
        }
        // Fallback to basic summaries if Ollama is not available
        Ok(sections.iter().map(|s| format!("Summary of section: {}", s.content)).collect())
    }

    async fn generate_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        self.embedder.embed_query(text)
            .await
            .map_err(|e: langchain_rust::embedding::EmbedderError| nt_core::Error::External(anyhow::anyhow!("Failed to generate embeddings: {}", e)))
            .map(|embeddings| embeddings.into_iter().map(|x| x as f32).collect())
    }
}

impl fmt::Debug for LangChainModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("LangChainModel");
        debug.field("embedder", &"OpenAI");
        #[cfg(feature = "qdrant")]
        debug.field("qdrant_store", &self.qdrant_store.is_some());
        #[cfg(feature = "ollama")]
        debug.field("ollama_client", &self.ollama_client.is_some());
        debug.finish()
    }
} 