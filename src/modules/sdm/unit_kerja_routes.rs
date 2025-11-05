// src/modules/sdm/unit_kerja_routes.rs
use super::unit_kerja_handler as handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{
    middleware,
    routing::{get},
    Router,
};

pub fn unit_kerja_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/sdm/unit-kerja",
            get(handler::get_all_handler).post(handler::create_handler),
        )
        .route(
            "/sdm/unit-kerja/{id}",
            get(handler::get_by_id_handler)
                .put(handler::update_handler)
                .delete(handler::delete_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BASDM".to_string(),
        ])))
}