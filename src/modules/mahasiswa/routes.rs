// src/modules/mahasiswa/routes.rs
use super::handler;
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    Router, middleware,
    routing::{get, post, put},
};

pub fn mahasiswa_router() -> Router<DbPool> {
    // =========================================================
    // 1. RUTE DATA MAHASISWA
    // =========================================================
    let rute_mahasiswa = Router::new()
        .route(
            "/mahasiswa/template-csv",
            get(handler::download_mahasiswa_csv_template_handler),
        )
        .route(
            "/mahasiswa/import-csv",
            post(handler::import_mahasiswa_from_csv_handler).layer(middleware::from_fn(
                require_role(vec!["SUPER_ADMIN".to_string(), "STAF_AKADEMIK".to_string()]),
            )),
        )
        .route(
            "/mahasiswa",
            get(handler::get_all_mahasiswa_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                    "DOSEN".to_string(),
                ])))
                .post(handler::create_mahasiswa_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ]))),
        )
        .route(
            "/mahasiswa/{id}",
            get(handler::get_mahasiswa_by_id_handler)
                .put(handler::update_mahasiswa_handler)
                .delete(handler::delete_mahasiswa_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ]))),
        );

    // =========================================================
    // 2. RUTE MANAJEMEN ROMBEL (ROMBONGAN BELAJAR)
    // =========================================================
    let rute_rombel = Router::new()
        .route("/akademik/rombel", get(handler::get_rombel_summary_handler))
        .route(
            "/akademik/rombel/mahasiswa",
            get(handler::get_mahasiswa_by_rombel_handler),
        )
        .route(
            "/akademik/rombel/pindah",
            put(handler::pindah_rombel_handler),
        )
        .route(
            "/akademik/rombel/rename",
            put(handler::rename_rombel_handler),
        )
        .layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_AKADEMIK".to_string(),
            "KAPRODI".to_string(),
        ])));

    // =========================================================
    // GABUNGKAN SEMUA RUTE MENJADI SATU KESATUAN MODUL
    // =========================================================
    Router::new().merge(rute_mahasiswa).merge(rute_rombel)
}
