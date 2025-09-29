// src/modules/akademik/jadwal_kuliah_routes.rs
use super::jadwal_kuliah_handler;
use crate::{db::DbPool,modules::auth::middleware::require_role};
use axum::{middleware,Router, routing::{get,put}};


pub fn jadwal_kuliah_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/akademik/jadwal-kuliah",
            get(jadwal_kuliah_handler::get_all_jadwal_kuliah_handler)
                .post(jadwal_kuliah_handler::create_jadwal_kuliah_handler),
        )
        .route(
            "/akademik/jadwal-kuliah/{id}", // <-- RUTE BARU
            put(jadwal_kuliah_handler::update_jadwal_kuliah_handler)
                .delete(jadwal_kuliah_handler::delete_jadwal_kuliah_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_AKADEMIK".to_string(),
        ])))
    // Tambahkan otorisasi di sini nanti
}
