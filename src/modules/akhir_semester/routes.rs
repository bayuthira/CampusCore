use super::handler;
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub fn akhir_semester_router() -> Router<DbPool> {
    let admin = Router::new()
        .route(
            "/akademik/akhir-semester/{id}",
            get(handler::status_handler),
        )
        .route(
            "/akademik/akhir-semester/{id}/tutup",
            post(handler::close_handler),
        )
        .route("/akademik/feeder/outbox", get(handler::outbox_handler))
        .route(
            "/akademik/feeder/outbox/{id}/hasil",
            post(handler::feeder_result_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_AKADEMIK".to_string(),
        ])));
    let correction = Router::new()
        .route(
            "/akademik/koreksi-nilai",
            get(handler::corrections_handler).post(handler::submit_correction_handler),
        )
        .route(
            "/akademik/koreksi-nilai/{id}/review",
            post(handler::review_correction_handler),
        )
        .route(
            "/akademik/koreksi-nilai/{id}/terapkan",
            post(handler::apply_correction_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_AKADEMIK".to_string(),
            "KAPRODI".to_string(),
            "DOSEN".to_string(),
        ])));
    let student = Router::new()
        .route("/akademik/khs-saya", get(handler::khs_handler))
        .route("/akademik/transkrip-saya", get(handler::transcript_handler))
        .route_layer(middleware::from_fn(require_role(vec![
            "MAHASISWA".to_string()
        ])));
    Router::new().merge(admin).merge(correction).merge(student)
}
