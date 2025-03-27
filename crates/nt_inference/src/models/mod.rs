use std::sync::Arc;
use nt_core::{Result, InferenceModel};
use crate::Config;

pub mod deepseek;
pub mod langchain;

pub async fn create_model(config: Option<Config>) -> Result<Arc<dyn InferenceModel>> {
    let config = config.unwrap_or_default();
    let model_name = config.model_name.as_deref().unwrap_or("ollama");

    match model_name.to_lowercase().as_str() {
        "ollama" => {
            #[cfg(feature = "ollama")]
            {
                let model = langchain::LangChainModel::new(Some(config)).await?;
                Ok(Arc::new(model))
            }
            #[cfg(not(feature = "ollama"))]
            {
                Err(nt_core::Error::Inference("Ollama support not enabled. Please enable the 'ollama' feature.".to_string()))
            }
        }
        "deepseek" => {
            let model = deepseek::DeepSeekModel::new(config.api_key)?;
            Ok(Arc::new(model))
        }
        _ => Err(nt_core::Error::Inference(format!("Unknown model: {}. Available models: ollama, deepseek", model_name))),
    }
} 