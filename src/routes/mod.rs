// src/routes/mod.rs

use crate::{auth::auth_middleware, db::DbPool, handlers};
use axum::{middleware, routing::{get, post}, Router};

// Deklarasikan semua modul rute baru kita
mod dosen_pa_routes;
mod dosen_routes;
mod krs_routes;
mod lookup_routes;
mod mahasiswa_routes;
mod matakuliah_routes;
mod prodi_routes;
mod tahun_akademik_routes;
mod user_management_routes;

/// Fungsi utama untuk membuat dan menggabungkan semua router
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

    // Gabungkan semua rute yang butuh proteksi login,
    // lalu terapkan middleware otentikasi utama sebagai lapisan terluar.
    let protected_routes = Router::new()
        .merge(dosen_pa_routes::dosen_pa_router())
        .merge(dosen_routes::dosen_router())
        .merge(krs_routes::krs_router())
        .merge(lookup_routes::lookup_router())
        .merge(mahasiswa_routes::mahasiswa_router())
        .merge(matakuliah_routes::matakuliah_router())
        .merge(prodi_routes::prodi_router())
        .merge(tahun_akademik_routes::tahun_akademik_router())
        .merge(user_management_routes::user_management_router())
        .route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware));

    // Gabungkan router publik dan terproteksi menjadi satu
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(pool)
}