use std::sync::Arc;
use nt_inference::InferenceModel;

pub struct AppState {
    pub inference_model: Arc<dyn InferenceModel>,
} 