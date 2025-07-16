// src/modules/health/routes.rs
use super::handler;
use crate::db::DbPool;
use axum::{routing::get, Router};

pub fn health_router() -> Router<DbPool> {
    // Tambahkan <DbPool> untuk memberitahu tipe state router ini
    Router::<DbPool>::new().route("/health", get(handler::health_check))
}