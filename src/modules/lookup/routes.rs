use crate::{db::DbPool, handlers};
use axum::{routing::get, Router};

pub fn lookup_router() -> Router<DbPool> {
    Router::new().route(
        "/api/lookups/enrollment-statuses",
        get(handlers::lookup_handler::get_enrollment_statuses_handler),
    )
}