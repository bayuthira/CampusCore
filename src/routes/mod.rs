use crate::{
    modules::auth::middleware::auth_middleware, // <-- Path baru yang benar
    db::DbPool,
    modules,
};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

// Tidak perlu deklarasi `mod` di sini karena sudah diatur di `main.rs` dan `modules/mod.rs`

/// Fungsi utama untuk membuat dan menggabungkan semua router
pub fn create_router(pool: DbPool) -> Router {
    // Rute publik (tidak perlu login)
    let public_routes = Router::new()
        // Kita panggil handler langsung dari path modularnya
        //.route("/health", get(modules::health::handler::health_check))
        .route("/auth/login", post(modules::auth::handler::login_handler));

    // Untuk saat ini, kita hanya gabungkan modul prodi yang sudah direfactor
    let protected_routes = Router::<DbPool>::new() // <-- Perlu tipe state <DbPool>
        .merge(modules::prodi::routes::prodi_router())
        // Anda bisa tambahkan .merge() untuk modul lain di sini nanti
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ));

    // Gabungkan router publik dan terproteksi menjadi satu dengan prefix /api
    Router::<DbPool>::new() // <-- Perlu tipe state <DbPool>
        .nest("/api", public_routes.merge(protected_routes))
        .with_state(pool)
}