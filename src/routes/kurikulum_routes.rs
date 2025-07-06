// src/routes/kurikulum_routes.rs

use crate::{auth::require_role, db::DbPool, handlers};
use axum::{middleware, routing::{delete, get, post, put}, Router};

pub fn kurikulum_router() -> Router<DbPool> {
    // Rute yang hanya bisa diakses admin/kaprodi
    let admin_routes = Router::new()
        .route("/api/kurikulum", post(handlers::kurikulum_handler::create_kurikulum_handler))
        .route(
            "/api/kurikulum/{id}", // <-- PERBAIKAN SINTAKS
            put(handlers::kurikulum_handler::update_kurikulum_handler)
                .delete(handlers::kurikulum_handler::delete_kurikulum_handler)
        )
        .route(
            "/api/kurikulum/{id}/matakuliah", // <-- PERBAIKAN SINTAKS
            post(handlers::kurikulum_handler::add_matakuliah_to_kurikulum_handler)
        )
        .route(
            "/api/kurikulum/{id}/matakuliah/{mk_id}", // <-- PERBAIKAN SINTAKS
            delete(handlers::kurikulum_handler::remove_matakuliah_from_kurikulum_handler)
        )
        .layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string(), "KAPRODI".to_string()])));

    // Rute yang bisa diakses semua user terotentikasi
    let all_user_routes = Router::new()
        .route("/api/kurikulum", get(handlers::kurikulum_handler::get_all_kurikulum_handler))
        .route(
            "/api/kurikulum/{id}", // <-- PERBAIKAN SINTAKS
            get(handlers::kurikulum_handler::get_kurikulum_by_id_handler)
        )
        .route(
            "/api/kurikulum/{id}/matakuliah", // <-- PERBAIKAN SINTAKS
            get(handlers::kurikulum_handler::get_matakuliah_in_kurikulum_handler)
        );

    // Gabungkan kedua grup router
    Router::new().merge(admin_routes).merge(all_user_routes)
}