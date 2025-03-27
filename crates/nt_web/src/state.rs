use std::sync::Arc;
use nt_core::InferenceModel;

pub struct AppState {
    pub inference_model: Arc<dyn InferenceModel>,
} 