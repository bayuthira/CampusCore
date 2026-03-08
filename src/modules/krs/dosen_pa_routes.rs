// src/modules/krs/dosen_pa_routes.rs
use super::dosen_pa_handler;
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    Router, middleware,
    routing::{get, put},
};

pub fn dosen_pa_router() -> Router<DbPool> {
    // Rute khusus untuk Dosen PA melihat datanya
    let dosen_routes = Router::new()
        .route(
            "/dosen-pa/my-advisees",
            get(dosen_pa_handler::get_my_advisees_handler)
                .layer(middleware::from_fn(require_role(vec!["DOSEN".to_string()]))),
        )
        .route(
            "/dosen-pa/advisee-krs/{mahasiswa_id}",
            get(dosen_pa_handler::get_advisee_krs_handler).layer(middleware::from_fn(
                require_role(vec!["DOSEN".to_string(), "SUPER_ADMIN".to_string()]),
            )),
        );

    // Rute khusus untuk Admin / Kaprodi melakukan Assignment Dosen PA
    let admin_routes = Router::new()
        .route(
            "/dosen-pa/batch-assign",
            put(dosen_pa_handler::batch_assign_dosen_pa_handler),
        )
        .route(
            "/dosen-pa/single-assign",
            put(dosen_pa_handler::single_assign_dosen_pa_handler),
        )
        .layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "KAPRODI".to_string(), // Asumsi KAPRODI juga berhak mengatur dosen PA
            "STAF_AKADEMIK".to_string(),
        ])));

    Router::new().merge(dosen_routes).merge(admin_routes)
}
