use crate::{auth::require_role, db::DbPool, handlers};
use axum::{middleware, routing::{delete, get, post, put}, Router};

pub fn krs_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/api/krs/enrollments",
            post(handlers::krs_handler::create_enrollment_handler)
                .layer(middleware::from_fn(require_role(vec!["MAHASISWA".to_string()]))),
        )
        .route(
            "/api/krs/enrollments/{id}",
            delete(handlers::krs_handler::delete_enrollment_handler).layer(
                middleware::from_fn(require_role(vec![
                    "MAHASISWA".to_string(),
                    "SUPER_ADMIN".to_string(),
                ])),
            ),
        )
        .route(
            "/api/krs/my-enrollments",
            get(handlers::krs_handler::get_my_enrollments_handler)
                .layer(middleware::from_fn(require_role(vec!["MAHASISWA".to_string()]))),
        )
        .route(
            "/api/krs/enrollments/{id}/status",
            put(handlers::krs_handler::update_enrollment_status_handler).layer(
                middleware::from_fn(require_role(vec![
                    "DOSEN".to_string(),
                    "SUPER_ADMIN".to_string(),
                ])),
            ),
        )
}