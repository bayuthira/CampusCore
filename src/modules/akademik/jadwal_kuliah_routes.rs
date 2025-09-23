// src/modules/akademik/jadwal_kuliah_routes.rs
use super::jadwal_kuliah_handler;
use crate::db::DbPool;
use axum::{
    Router,
    routing::{get},
};

pub fn jadwal_kuliah_router() -> Router<DbPool> {
    Router::new().route(
        "/akademik/jadwal-kuliah",
        get(jadwal_kuliah_handler::get_all_jadwal_kuliah_handler)
            .post(jadwal_kuliah_handler::create_jadwal_kuliah_handler),
    )
    // Tambahkan otorisasi di sini nanti
}
