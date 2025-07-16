// src/routes/aset_routes.rs
use super::{handler as aset_handler,jenis_aset_handler,ruangan_handler};
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{Router, middleware, routing::{get,post}};

pub fn aset_router() -> Router<DbPool> {
    // Gabungkan semua rute untuk modul aset di sini
    Router::new()
        // Rute untuk Jenis Aset
        .route(
            "/aset/jenis",
            get(jenis_aset_handler::get_all_jenis_aset_handler)
                .post(jenis_aset_handler::create_jenis_aset_handler),
        )
        .route(
            "/aset/jenis/{id}",
            get(jenis_aset_handler::get_jenis_aset_by_id_handler)
                .put(jenis_aset_handler::update_jenis_aset_handler)
                .delete(jenis_aset_handler::delete_jenis_aset_handler),
        )
        .route(
            "/aset/ruangan",
            get(ruangan_handler::get_all_ruangan_handler)
                .post(ruangan_handler::create_ruangan_handler),
        )
        .route(
            "/aset/ruangan/{id}",
            get(ruangan_handler::get_ruangan_by_id_handler)
                .put(ruangan_handler::update_ruangan_handler)
                .delete(ruangan_handler::delete_ruangan_handler),
        )
        .route(
            "/aset/item",
            get(aset_handler::get_all_aset_handler)
                .post(aset_handler::create_aset_handler),
        )
        .route(
            "/aset/item/{id}",
            get(aset_handler::get_aset_by_id_handler)
                .put(aset_handler::update_aset_handler)
                .delete(aset_handler::delete_aset_handler),
        )
        .route(
            "/aset/item/{id}/histori", 
            get(aset_handler::get_aset_histori_handler)
        )
        .route(
            "/aset/item/{id}/pindahkan",
            post(aset_handler::pindahkan_aset_handler)
        )
        // Terapkan middleware untuk semua rute di atas
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BAUM".to_string(),
        ])))
}
