// src/modules/sdm/routes.rs

use super::handler;
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{Router, middleware, routing::{get,post,delete}};

pub fn sdm_router() -> Router<DbPool> {
    // Grup 1: Aksi untuk STAF_BASDM & SUPER_ADMIN (Create, Read, Update)
    let sdm_staff_routes = Router::new()
        .route(
            "/sdm/pegawai",
            // GET untuk melihat daftar, POST untuk membuat data baru
            get(handler::get_all_pegawai_handler)
                .post(handler::create_pegawai_handler),
        )
        .route(
            "/sdm/pegawai/{id}",
            // GET untuk melihat detail, PUT untuk memperbarui
            get(handler::get_pegawai_by_id_handler)
                .put(handler::update_pegawai_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BASDM".to_string(),
        ])));

    // Grup 2: Aksi khusus SUPER_ADMIN (Delete, Create User)
    let super_admin_routes = Router::new()
        .route(
            "/sdm/pegawai/{id}",
            // DELETE untuk menghapus data pegawai
            delete(handler::delete_pegawai_handler),
        )
        .route(
            "/sdm/pegawai/{id}/create-user",
            // POST untuk membuatkan akun login
            post(handler::create_user_for_pegawai_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
        ])));

    // Gabungkan kedua grup menjadi satu router untuk modul SDM
    Router::new().merge(sdm_staff_routes).merge(super_admin_routes)
}

