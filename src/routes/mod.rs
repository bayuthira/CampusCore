use crate::{db::DbPool, modules, modules::auth::middleware::auth_middleware};
use axum::{
    Router, middleware,
    routing::{get, post},
};

/// Fungsi utama untuk membuat dan menggabungkan semua router
pub fn create_router(pool: DbPool) -> Router {
    // Rute publik (tidak perlu login)
    let public_routes = Router::new()
        .merge(modules::health::routes::health_router())
        .route("/auth/login", post(modules::auth::handler::login_handler));

    // Rute Login
    let protected_routes = Router::<DbPool>::new() // <-- Perlu tipe state <DbPool>
        .merge(modules::prodi::routes::prodi_router())
        .merge(modules::dosen::routes::dosen_router())
        .merge(modules::mahasiswa::routes::mahasiswa_router())
        .merge(modules::matakuliah::routes::matakuliah_router())
        .merge(modules::tahun_akademik::routes::tahun_akademik_router())
        .merge(modules::kurikulum::routes::kurikulum_router())
        .merge(modules::krs::dosen_pa_routes::dosen_pa_router())
        .merge(modules::krs::routes::krs_router())
        .merge(modules::user_management::routes::user_management_router())
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ));

    Router::<DbPool>::new()
        .nest("/api", public_routes.merge(protected_routes))
        .with_state(pool)
}
