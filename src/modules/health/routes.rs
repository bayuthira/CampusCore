// src/modules/health/routes.rs
use super::handler;
use axum::{routing::get, Router};

pub fn health_router() -> Router {
    Router::new().route("/health", get(handler::health_check))
}