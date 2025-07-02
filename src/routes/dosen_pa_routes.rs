use crate::{auth::require_role, db::DbPool, handlers};
use axum::{middleware, routing::get, Router};

pub fn dosen_pa_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/api/dosen-pa/my-advisees",
            get(handlers::dosen_pa_handler::get_my_advisees_handler)
                .layer(middleware::from_fn(require_role(vec!["DOSEN".to_string()]))),
        )
        .route(
            "/api/dosen-pa/advisee-krs/{mahasiswa_id}",
            get(handlers::dosen_pa_handler::get_advisee_krs_handler).layer(
                middleware::from_fn(require_role(vec![
                    "DOSEN".to_string(),
                    "SUPER_ADMIN".to_string(),
                ])),
            ),
        )
}