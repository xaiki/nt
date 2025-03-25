use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub mod handlers;
pub mod state;
pub mod routes;

pub use state::AppState;

pub async fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::permissive();
    
    Router::new()
        .route("/api/articles", get(handlers::list_articles))
        .route("/api/articles", post(handlers::create_article))
        .route("/api/articles/:id", get(handlers::get_article))
        .route("/api/articles/:id/similar", get(handlers::get_similar_articles))
        .route("/api/articles/:id/divergence", get(handlers::get_article_divergence))
        .layer(cors)
        .with_state(Arc::new(state))
}

pub mod prelude {
    pub use nt_core::{Article, Result, Error};
    pub use crate::AppState;
} 