use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
};
use std::sync::Arc;
use nt_core::Article;
use crate::AppState;
use serde_json::Value;
use chrono::Utc;

pub async fn list_articles(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Json::<Vec<Article>>(vec![])
}

pub async fn create_article(
    State(_state): State<Arc<AppState>>,
    Json(_article): Json<Article>,
) -> impl IntoResponse {
    let default_article = Article {
        url: "".to_string(),
        title: "".to_string(),
        content: "".to_string(),
        published_at: Utc::now(),
        source: "".to_string(),
        sections: vec![],
        summary: None,
        authors: vec![],
        related_articles: Vec::new(),
    };
    Json(default_article)
}

pub async fn get_article(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    let default_article = Article {
        url: "".to_string(),
        title: "".to_string(),
        content: "".to_string(),
        published_at: Utc::now(),
        source: "".to_string(),
        sections: vec![],
        summary: None,
        authors: vec![],
        related_articles: Vec::new(),
    };
    Json(default_article)
}

pub async fn get_similar_articles(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    Json::<Vec<Article>>(vec![])
}

pub async fn get_article_divergence(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> impl IntoResponse {
    Json(Value::Null)
} 