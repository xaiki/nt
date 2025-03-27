use crate::{Result, Config};
use nt_core::InferenceModel;
use std::sync::Arc;

pub mod deepseek;
#[cfg(any(feature = "qdrant", feature = "sqlite"))]
pub mod langchain;

pub fn create_model(config: Option<Config>) -> Result<Arc<dyn InferenceModel>> {
    // For now, we'll use DeepSeek as the default model
    // In the future, this could be configurable based on the config parameter
    let api_key = config.and_then(|c| c.api_key);
    Ok(Arc::new(deepseek::DeepSeekModel::new(api_key)?))
} 