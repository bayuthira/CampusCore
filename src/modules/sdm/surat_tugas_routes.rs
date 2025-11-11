// src/modules/sdm/surat_tugas_routes.rs
use super::surat_tugas_handler as handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{
    middleware,
    routing::{get},
    Router,
};

pub fn surat_tugas_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/sdm/surat-tugas",
            get(handler::get_all_surat_tugas_handler)
                .post(handler::create_surat_tugas_handler),
        )
        .route(
            "/sdm/surat-tugas/{id}",
            get(handler::get_surat_tugas_detail_handler)
                .put(handler::update_surat_tugas_handler)
                .delete(handler::delete_surat_tugas_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BASDM".to_string(),
        ])))
}