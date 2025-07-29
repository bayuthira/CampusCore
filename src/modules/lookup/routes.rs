use crate::{db::DbPool};
use super::handler;
use axum::{routing::get, Router};

pub fn lookup_router() -> Router<DbPool> {
    Router::new().route(
        "/lookups/enrollment-statuses",
        get(handler::get_enrollment_statuses_handler),
    )
    .route(
            "/lookups/kondisi-aset",
            get(handler::get_kondisi_aset_handler),
        )
}