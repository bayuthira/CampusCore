// src/routes/prodi_routes.rs

use crate::{auth::require_role, db::DbPool, handlers};
use axum::{
    handler::Handler,
    middleware,
    routing::{get},
    Router,
};

pub fn prodi_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/api/prodi",
            // GET bisa diakses semua user terotentikasi
            get(handlers::prodi_handler::get_all_prodi_handler)
            // POST hanya oleh SUPER_ADMIN
            .post(
                handlers::prodi_handler::create_prodi_handler
                    .layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()]))),
            ),
        )
        // Tambahkan rute untuk operasi by ID
        .route(
            "/api/prodi/{id}",
            get(handlers::prodi_handler::get_prodi_by_id_handler)
                .put(handlers::prodi_handler::update_prodi_handler)
                .delete(handlers::prodi_handler::delete_prodi_handler)
        )
        // Semua operasi di atas (kecuali get all) hanya untuk SUPER_ADMIN
        .layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()])))
}