// src/routes/mod.rs
use crate::{db::DbPool, handlers};
use axum::{
    Router,
    routing::{delete, get, post, put}, // Import 'post'
};

// Fungsi untuk membuat router aplikasi dengan state DbPool
pub fn create_router(pool: DbPool) -> Router {
    Router::new()
        // Rute untuk health check
        .route("/api/health", get(handlers::health_handler::health_check))
        // Tambahkan rute lain di sini nanti
        // Rute baru Program Studi
        .route(
            "/api/prodi",
            post(handlers::prodi_handler::create_prodi_handler),
        )
        .route(
            "/api/prodi",
            get(handlers::prodi_handler::get_all_prodi_handler),
        )
        // Rute untuk Dosen
        .route(
            "/api/dosen",
            post(handlers::dosen_handler::create_dosen_handler),
        )
        .route(
            "/api/dosen",
            get(handlers::dosen_handler::get_all_dosen_handler),
        )
        .route(
            "/api/dosen/{id}",
            get(handlers::dosen_handler::get_dosen_by_id_handler),
        )
        .route(
            "/api/dosen/{id}",
            put(handlers::dosen_handler::update_dosen_handler),
        )
        .route(
            "/api/dosen/{id}",
            delete(handlers::dosen_handler::delete_dosen_handler),
        )
        // .route("/api/users", post(...))
        .with_state(pool) // Menyediakan pool database sebagai state untuk semua handler
}
