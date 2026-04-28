// src/modules/matakuliah/routes.rs
use super::{handler, rps_handler};
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    Router,
    handler::Handler,
    middleware,
    routing::{delete, get, put},
};

pub fn matakuliah_router() -> Router<DbPool> {
    let base_routes =
        Router::new()
            .route(
                "/matakuliah",
                get(handler::get_all_matakuliah_handler).post(
                    handler::create_matakuliah_handler.layer(middleware::from_fn(require_role(
                        vec!["SUPER_ADMIN".to_string(), "KAPRODI".to_string()],
                    ))),
                ),
            )
            .route(
                "/matakuliah/{id}",
                get(handler::get_matakuliah_by_id_handler)
                    .put(handler::update_matakuliah_handler)
                    .delete(handler::delete_matakuliah_handler)
                    .layer(middleware::from_fn(require_role(vec![
                        "SUPER_ADMIN".to_string(),
                        "KAPRODI".to_string(),
                    ]))),
            )
            // --- RUTE VERIFIKASI RPS YANG KETINGGALAN ---
            .route(
                "/matakuliah/{id}/verifikasi-rps",
                put(handler::verifikasi_rps_handler).layer(middleware::from_fn(require_role(
                    vec!["SUPER_ADMIN".to_string(), "KAPRODI".to_string()],
                ))),
            );

    // Grup rute khusus RPS Terstruktur
    let rps_routes = Router::new()
        .route(
            "/matakuliah/{id}/rps-header",
            get(rps_handler::get_rps_header_handler).put(rps_handler::upsert_rps_header_handler),
        )
        .route(
            "/matakuliah/{id}/rps-mingguan",
            get(rps_handler::get_rps_mingguan_handler)
                .post(rps_handler::upsert_rps_mingguan_handler),
        )
        .route(
            "/matakuliah/rps-mingguan/{id_mingguan}",
            delete(rps_handler::delete_rps_mingguan_handler),
        )
        .layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "KAPRODI".to_string(),
            "DOSEN".to_string(), // Dosen diperbolehkan mengedit RPS jika mereka Koordinator
        ])));

    Router::new().merge(base_routes).merge(rps_routes)
}
