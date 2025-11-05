// src/modules/sdm/penempatan_routes.rs
use super::penempatan_handler as handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{
    middleware,
    routing::{get, put},
    Router,
};

pub fn penempatan_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/sdm/pegawai/{pegawai_id}/penempatan",
            get(handler::get_all_by_pegawai_id_handler)
                .post(handler::create_handler),
        )
        .route(
            "/sdm/penempatan/{id}",
            put(handler::update_handler)
                .delete(handler::delete_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BASDM".to_string(),
        ])))
}