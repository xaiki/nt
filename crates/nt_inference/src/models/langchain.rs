use std::sync::Arc;
use std::fmt;
use nt_core::{Result, Article, ArticleSection};
use super::{InferenceModel, Config};
use nt_storage::{BackendConfig, EmbeddingModel};
#[cfg(feature = "qdrant")]
use nt_storage::backends::qdrant::{QdrantConfig, QdrantStore};
#[cfg(feature = "sqlite")]
use nt_storage::backends::sqlite::{SQLiteConfig, SQLiteStore};

#[cfg(feature = "qdrant")]
use langchain_rust::{
    embedding::{openai::{OpenAiEmbedder, OpenAIConfig}, Embedder},
    vectorstore::qdrant::{Qdrant, StoreBuilder, Store},
    vectorstore::VectorStore,
};

#[cfg(feature = "sqlite")]
use langchain_rust::{
    embedding::sqlite::SQLiteEmbeddings,
    vectorstore::sqlite::SQLiteStore,
};

#[cfg(feature = "ollama")]
use langchain_rust::{
    embedding::ollama::OllamaEmbeddings,
    llm::ollama::client::Ollama,
};

pub struct LangChainModel {
    embedder: Arc<OpenAiEmbedder<OpenAIConfig>>,
    #[cfg(feature = "qdrant")]
    qdrant_store: Option<Arc<Store>>,
    #[cfg(feature = "sqlite")]
    sqlite_store: Option<Arc<SQLiteStore>>,
    #[cfg(feature = "ollama")]
    ollama_client: Option<Arc<Ollama>>,
}

impl LangChainModel {
    pub fn new(_api_key: Option<String>) -> Result<Self> {
        let config = OpenAIConfig::default();
        let embedder = Arc::new(OpenAiEmbedder::new(config));

        #[cfg(feature = "qdrant")]
        let qdrant_store = None;

        #[cfg(feature = "sqlite")]
        let sqlite_store = None;

        #[cfg(feature = "ollama")]
        let ollama_client = Some(Arc::new(Ollama::default().with_model("llama2")));

        Ok(Self {
            embedder,
            #[cfg(feature = "qdrant")]
            qdrant_store,
            #[cfg(feature = "sqlite")]
            sqlite_store,
            #[cfg(feature = "ollama")]
            ollama_client,
        })
    }

    pub async fn with_backend_config<T: BackendConfig>(mut self, config: &T) -> Result<Self> {
        match config.get_embedding_model() {
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
                let client = Qdrant::from_url(&qdrant_config.get_url())
                    .build()
                    .map_err(|e| nt_core::Error::External(anyhow::anyhow!("Failed to build Qdrant client: {}", e)))?;

                let embedder = OpenAiEmbedder::new(OpenAIConfig::default());
                let store = StoreBuilder::new()
                    .embedder(embedder)
                    .client(client)
                    .collection_name(&qdrant_config.get_collection())
                    .build()
                    .await
                    .map_err(|e| nt_core::Error::External(anyhow::anyhow!("Failed to build Qdrant store: {}", e)))?;

                self.qdrant_store = Some(Arc::new(store));
                Ok(self)
            }
            #[cfg(feature = "sqlite")]
            EmbeddingModel::SQLite => {
                let sqlite_config = SQLiteConfig::new();
                let embeddings = Arc::new(SQLiteEmbeddings::new(
                    sqlite_config.get_url(),
                    sqlite_config.get_collection(),
                ));
                self.embedder = embeddings;
                Ok(self)
            }
            #[cfg(not(feature = "qdrant"))]
            EmbeddingModel::Qdrant => {
                Err(nt_core::Error::External(anyhow::anyhow!("Qdrant feature not enabled")))
            }
            #[cfg(not(feature = "sqlite"))]
            EmbeddingModel::SQLite => {
                Err(nt_core::Error::External(anyhow::anyhow!("SQLite feature not enabled")))
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
        #[cfg(feature = "sqlite")]
        debug.field("sqlite_store", &self.sqlite_store.is_some());
        #[cfg(feature = "ollama")]
        debug.field("ollama_client", &self.ollama_client.is_some());
        debug.finish()
    }
} 