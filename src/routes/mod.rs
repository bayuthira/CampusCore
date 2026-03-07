use crate::{db::DbPool, modules, modules::auth::middleware::auth_middleware};
use axum::{
    Router, middleware,
    routing::{get, post},
};
// use tower_http::services::ServeDir;

/// Fungsi utama untuk membuat dan menggabungkan semua router
pub fn create_router(pool: DbPool) -> Router {
    // Rute publik (tidak perlu login)
    let public_routes = Router::new()
        .merge(modules::health::routes::health_router())
        .route("/auth/login", post(modules::auth::handler::login_handler));

    // Rute Login
    let protected_routes = Router::<DbPool>::new() // <-- Perlu tipe state <DbPool>
        .merge(modules::lookup::routes::lookup_router())
        .merge(modules::prodi::routes::prodi_router())
        .merge(modules::dosen::routes::dosen_router())
        .merge(modules::mahasiswa::routes::mahasiswa_router())
        .merge(modules::matakuliah::routes::matakuliah_router())
        .merge(modules::tahun_akademik::routes::tahun_akademik_router())
        .merge(modules::kurikulum::routes::kurikulum_router())
        //   .merge(modules::krs::dosen_pa_routes::dosen_pa_router())
        //   .merge(modules::krs::routes::krs_router())
        .merge(modules::user_management::routes::user_management_router())
        .merge(modules::aset::routes::aset_router())
        .merge(modules::akademik::jadwal_kuliah_routes::jadwal_kuliah_router())
        .merge(modules::fleet::routes::fleet_router())
        .merge(modules::fleet::servis_routes::servis_router())
        .merge(modules::sdm::routes::sdm_router())
        .route(
            "/files/{*path}",
            get(modules::files::handler::serve_file_handler),
        )
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ));

    Router::<DbPool>::new()
        .nest("/api", public_routes.merge(protected_routes))
        //    .nest_service("/uploads", ServeDir::new("uploads"))
        .with_state(pool)
}
