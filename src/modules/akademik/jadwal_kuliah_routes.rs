// src/modules/akademik/jadwal_kuliah_routes.rs
use super::jadwal_kuliah_handler;
use crate::db::DbPool;
use axum::{routing::post, Router};

pub fn jadwal_kuliah_router() -> Router<DbPool> {
    Router::new().route("/akademik/jadwal-kuliah", post(jadwal_kuliah_handler::create_jadwal_kuliah_handler))    
    // Tambahkan otorisasi di sini nanti
}