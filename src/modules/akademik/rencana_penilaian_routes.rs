// src/modules/akademik/rencana_penilaian_routes.rs
use super::rencana_penilaian_handler as handler;
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    Router, middleware,
    routing::{get, post},
};

pub fn rencana_penilaian_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/akademik/jadwal-kuliah/{jadwal_kuliah_id}/rencana-penilaian",
            get(handler::get_rencana_penilaian_handler)
                .put(handler::upsert_rencana_penilaian_handler),
        )
        .route(
            "/akademik/jadwal-kuliah/{jadwal_kuliah_id}/rencana-penilaian/upload/{jenis_file}",
            post(handler::upload_file_rencana_penilaian_handler),
        )
        .layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "KAPRODI".to_string(),
            "DOSEN".to_string(),
        ])))
}
