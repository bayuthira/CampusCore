use super::handler;
use crate::{
    modules::auth::middleware::require_role,
    db::DbPool,
};
use axum::{middleware, routing::{delete, get, post, put}, Router};

pub fn krs_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/krs/enrollments",
            post(handler::create_enrollment_handler)
                .layer(middleware::from_fn(require_role(vec!["MAHASISWA".to_string()]))),
        )
        .route(
            "/krs/enrollments/{id}",
            delete(handler::delete_enrollment_handler).layer(
                middleware::from_fn(require_role(vec![
                    "MAHASISWA".to_string(),
                    "SUPER_ADMIN".to_string(),
                ])),
            ),
        )
        .route(
            "/krs/my-enrollments",
            get(handler::get_my_enrollments_handler)
                .layer(middleware::from_fn(require_role(vec!["MAHASISWA".to_string()]))),
        )
        .route(
            "/krs/enrollments/{id}/status",
            put(handler::update_enrollment_status_handler).layer(
                middleware::from_fn(require_role(vec![
                    "DOSEN".to_string(),
                    "SUPER_ADMIN".to_string(),
                ])),
            ),
        )
}