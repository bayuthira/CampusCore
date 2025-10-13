// src/modules/sdm/routes.rs

use super::handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{
    middleware,
    routing::{get},
    Router,
};

pub fn sdm_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/sdm/pegawai",
            get(handler::get_all_pegawai_handler)
                .post(handler::create_pegawai_handler),
        )
        .route(
            "/sdm/pegawai/{id}",
            get(handler::get_pegawai_by_id_handler)
                .put(handler::update_pegawai_handler)
                .delete(handler::delete_pegawai_handler),
        )
        // Semua endpoint di atas hanya bisa diakses oleh SUPER_ADMIN
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
        ])))
}