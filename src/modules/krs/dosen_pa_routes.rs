use super::dosen_pa_handler;
use crate::{
    modules::auth::middleware::require_role,
    db::DbPool,
};
use axum::{middleware, routing::get, Router};

pub fn dosen_pa_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/dosen-pa/my-advisees",
            get(dosen_pa_handler::get_my_advisees_handler)
                .layer(middleware::from_fn(require_role(vec!["DOSEN".to_string()]))),
        )
        .route(
            "/dosen-pa/advisee-krs/{mahasiswa_id}",
            get(dosen_pa_handler::get_advisee_krs_handler).layer(
                middleware::from_fn(require_role(vec![
                    "DOSEN".to_string(),
                    "SUPER_ADMIN".to_string(),
                ])),
            ),
        )
}