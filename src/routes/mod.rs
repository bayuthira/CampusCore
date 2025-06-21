// src/routes/mod.rs

// Perbaiki `use` statement agar benar
use crate::{
    auth::{auth_middleware, require_role},
    db::DbPool,
    handlers,
};
use axum::{
    Router,
    handler::Handler, // <-- Ini penting untuk `.layer()`
    middleware,
    routing::{get, post}, // <-- Hanya import yang kita pakai
};

pub fn create_router(pool: DbPool) -> Router {
    // Rute publik
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

    // Handler untuk membuat prodi, sudah dilapisi middleware otorisasi
    let create_prodi_handler = handlers::prodi_handler::create_prodi_handler.layer(
        middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()])),
    );

    // Rute yang butuh login
    let protected_routes = Router::new()
        .route(
            "/api/prodi",
            get(handlers::prodi_handler::get_all_prodi_handler).post(create_prodi_handler), // Gunakan handler yang sudah di-layer
        )
        // Gabungkan semua method untuk /api/dosen
        .route(
            "/api/dosen",
            get(handlers::dosen_handler::get_all_dosen_handler)
                .post(handlers::dosen_handler::create_dosen_handler),
        )
        // Gabungkan semua method untuk /api/dosen/{id}
        .route(
            "/api/dosen/{id}",
            get(handlers::dosen_handler::get_dosen_by_id_handler)
                .put(handlers::dosen_handler::update_dosen_handler)
                .delete(handlers::dosen_handler::delete_dosen_handler),
        )
        // --- Rute untuk Mahasiswa ---
        .route(
            "/api/mahasiswa",
            // GET bisa diakses oleh lebih banyak peran (misal: DOSEN juga bisa lihat)
            get(handlers::mahasiswa_handler::get_all_mahasiswa_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                    "DOSEN".to_string(),
                ])))
                // POST hanya bisa diakses oleh SUPER_ADMIN dan STAF_AKADEMIK
                .post(handlers::mahasiswa_handler::create_mahasiswa_handler.layer(
                    middleware::from_fn(require_role(vec![
                        "SUPER_ADMIN".to_string(),
                        "STAF_AKADEMIK".to_string(),
                    ])),
                )),
        )
        .route(
            "/api/mahasiswa/import-csv", // Rute baru kita
            post(handlers::mahasiswa_handler::import_mahasiswa_from_csv_handler).layer(
                middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "STAF_AKADEMIK".to_string(),
                ])),
            ),
        )
        // Terapkan middleware otentikasi ke SEMUA rute di atas
        .route_layer(middleware::from_fn(auth_middleware));

    // Gabungkan semua router
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(pool)
}
