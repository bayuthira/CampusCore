// src/routes/mod.rs

use crate::{
    auth::{auth_middleware, require_role},
    db::DbPool,
    handlers,
};
use axum::{
    Router,
    handler::Handler,
    middleware,
    routing::{delete, get, post, put},
};

pub fn create_router(pool: DbPool) -> Router {
    // Rute publik (tidak perlu login)
    let public_routes = Router::new()
        .route("/api/health", get(handlers::health_handler::health_check))
        .route(
            "/api/auth/register",
            post(handlers::auth_handler::register_handler),
        )
        .route(
            "/api/auth/login",
            post(handlers::auth_handler::login_handler),
        );

    // Rute yang butuh login
    let protected_routes = Router::new()
        // --- Rute untuk Lookup ---
        .route(
            "/api/lookups/enrollment-statuses",
            get(handlers::lookup_handler::get_enrollment_statuses_handler),
        )
        // --- Rute untuk Program Studi ---
        .route(
            "/api/prodi",
            get(handlers::prodi_handler::get_all_prodi_handler).post(
                handlers::prodi_handler::create_prodi_handler.layer(middleware::from_fn(
                    require_role(vec!["SUPER_ADMIN".to_string()]),
                )),
            ),
        )
        // --- Rute untuk Dosen ---
        .route(
            "/api/dosen",
            get(handlers::dosen_handler::get_all_dosen_handler)
                .post(handlers::dosen_handler::create_dosen_handler),
        )
        .route(
            "/api/dosen/{id}",
            get(handlers::dosen_handler::get_dosen_by_id_handler)
                .put(handlers::dosen_handler::update_dosen_handler)
                .delete(handlers::dosen_handler::delete_dosen_handler),
        )
        // --- Rute untuk Mahasiswa ---
        .route(
            "/api/mahasiswa/import-csv",
            post(handlers::mahasiswa_handler::import_mahasiswa_from_csv_handler).layer(
                middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ])),
            ),
        )
        .route(
            "/api/mahasiswa",
            get(handlers::mahasiswa_handler::get_all_mahasiswa_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                    "DOSEN".to_string(),
                ])))
                .post(handlers::mahasiswa_handler::create_mahasiswa_handler.layer(
                    middleware::from_fn(require_role(vec![
                        "SUPER_ADMIN".to_string(),
                        "STAF_AKADEMIK".to_string(),
                    ])),
                )),
        )
        .route(
            "/api/mahasiswa/{id}",
            get(handlers::mahasiswa_handler::get_mahasiswa_by_id_handler)
                .put(handlers::mahasiswa_handler::update_mahasiswa_handler)
                .delete(handlers::mahasiswa_handler::delete_mahasiswa_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ]))),
        )
        // --- Rute untuk Mata Kuliah ---
        .route(
            "/api/matakuliah",
            get(handlers::matakuliah_handler::get_all_matakuliah_handler).post(
                handlers::matakuliah_handler::create_matakuliah_handler.layer(middleware::from_fn(
                    require_role(vec!["SUPER_ADMIN".to_string(), "KAPRODI".to_string()]),
                )),
            ),
        )
        .route(
            "/api/matakuliah/{id}",
            get(handlers::matakuliah_handler::get_matakuliah_by_id_handler)
                .put(handlers::matakuliah_handler::update_matakuliah_handler)
                .delete(handlers::matakuliah_handler::delete_matakuliah_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "KAPRODI".to_string(),
                ]))),
        )
        // --- Rute untuk Tahun Akademik (Hanya SUPER_ADMIN) ---  <-- INI YANG HILANG
        .route(
            "/api/tahun-akademik",
            get(handlers::tahun_akademik_handler::get_all_tahun_akademik_handler)
                .post(handlers::tahun_akademik_handler::create_tahun_akademik_handler),
        )
        .route(
            "/api/tahun-akademik/{id}",
            get(handlers::tahun_akademik_handler::get_tahun_akademik_by_id_handler)
                .put(handlers::tahun_akademik_handler::update_tahun_akademik_handler)
                .delete(handlers::tahun_akademik_handler::delete_tahun_akademik_handler),
        )
        .layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
        ])))
        // --- Rute untuk KRS ---
        .route(
            "/api/krs/enrollments",
            post(handlers::krs_handler::create_enrollment_handler).layer(middleware::from_fn(
                require_role(vec!["MAHASISWA".to_string()]),
            )),
        )
        .route(
            "/api/krs/enrollments/{id}",
            delete(handlers::krs_handler::delete_enrollment_handler)
                // Izinkan MAHASISWA dan SUPER_ADMIN untuk mengakses endpoint ini
                .layer(middleware::from_fn(require_role(vec![
                    "MAHASISWA".to_string(),
                    "SUPER_ADMIN".to_string(),
                ]))),
        )
        .route(
            "/api/krs/my-enrollments",
            get(handlers::krs_handler::get_my_enrollments_handler).layer(middleware::from_fn(
                require_role(vec!["MAHASISWA".to_string()]),
            )),
        )
        .route(
            "/api/krs/enrollments/{id}/status", // <-- RUTE BARU KITA
            put(handlers::krs_handler::update_enrollment_status_handler)
                // Hanya DOSEN dan SUPER_ADMIN yang bisa mencoba mengakses endpoint ini
                .layer(middleware::from_fn(require_role(vec![
                    "DOSEN".to_string(),
                    "SUPER_ADMIN".to_string(),
                ]))),
        )
        // Terapkan middleware otentikasi utama ke SEMUA rute di grup ini
        .route_layer(middleware::from_fn(auth_middleware));

    // Gabungkan semua router
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(pool)
}
