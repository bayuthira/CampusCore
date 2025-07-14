// src/routes/aset_routes.rs
use crate::{auth::require_role, db::DbPool, handlers};
use axum::{Router, middleware, routing::get};

pub fn aset_router() -> Router<DbPool> {
    // Gabungkan semua rute untuk modul aset di sini
    Router::new()
        // Rute untuk Jenis Aset
        .route(
            "/api/aset/jenis",
            get(handlers::jenis_aset_handler::get_all_jenis_aset_handler)
                .post(handlers::jenis_aset_handler::create_jenis_aset_handler),
        )
        .route(
            "/api/aset/jenis/{id}",
            get(handlers::jenis_aset_handler::get_jenis_aset_by_id_handler)
                .put(handlers::jenis_aset_handler::update_jenis_aset_handler)
                .delete(handlers::jenis_aset_handler::delete_jenis_aset_handler),
        )
        .route(
            "/api/aset/ruangan",
            get(handlers::ruangan_handler::get_all_ruangan_handler)
                .post(handlers::ruangan_handler::create_ruangan_handler),
        )
        .route(
            "/api/aset/ruangan/{id}",
            get(handlers::ruangan_handler::get_ruangan_by_id_handler)
                .put(handlers::ruangan_handler::update_ruangan_handler)
                .delete(handlers::ruangan_handler::delete_ruangan_handler),
        )
        .route(
            "/api/aset/item",
            get(handlers::aset_handler::get_all_aset_handler)
                .post(handlers::aset_handler::create_aset_handler),
        )
        .route(
            "/api/aset/item/{id}",
            get(handlers::aset_handler::get_aset_by_id_handler)
                .put(handlers::aset_handler::update_aset_handler)
                .delete(handlers::aset_handler::delete_aset_handler),
        )
        // Terapkan middleware untuk semua rute di atas
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BAUM".to_string(),
        ])))
}
