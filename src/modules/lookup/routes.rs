use super::handler;
use crate::db::DbPool;
use axum::{Router, routing::get};

pub fn lookup_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/lookups/enrollment-statuses",
            get(handler::get_enrollment_statuses_handler),
        )
        .route(
            "/lookups/kondisi-aset",
            get(handler::get_kondisi_aset_handler),
        )
        .route(
            "/lookups/aset-histori-statuses",
            get(handler::get_aset_histori_statuses_handler),
        )
}
