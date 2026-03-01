// src/routes/kurikulum/routes.rs
use super::handler;
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};

pub fn kurikulum_router() -> Router<DbPool> {
    // Rute yang hanya bisa diakses admin/kaprodi
    let admin_routes = Router::new()
        .route("/kurikulum", post(handler::create_kurikulum_handler))
        .route(
            "/kurikulum/{id}", // <-- PERBAIKAN SINTAKS
            put(handler::update_kurikulum_handler).delete(handler::delete_kurikulum_handler),
        )
        .route(
            "/kurikulum/{id}/matakuliah",
            post(handler::add_matakuliah_to_kurikulum_handler),
        )
        .route(
            "/kurikulum/{id}/matakuliah/{mk_id}",
            delete(handler::remove_matakuliah_from_kurikulum_handler),
        )
        .layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "KAPRODI".to_string(),
        ])));

    // Rute yang bisa diakses semua user terotentikasi
    let all_user_routes = Router::new()
        .route("/kurikulum", get(handler::get_all_kurikulum_handler))
        .route("/kurikulum/{id}", get(handler::get_kurikulum_by_id_handler))
        .route(
            "/kurikulum/{id}/matakuliah",
            get(handler::get_matakuliah_in_kurikulum_handler),
        );

    // Gabungkan kedua grup router
    Router::new().merge(admin_routes).merge(all_user_routes)
}
