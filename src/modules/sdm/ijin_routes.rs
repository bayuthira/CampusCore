use super::ijin_handler as handler;
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    Router, middleware,
    routing::{get, post, put},
};

pub fn ijin_router() -> Router<DbPool> {
    // Rute untuk pegawai (mengajukan dan melihat riwayat sendiri)
    let pegawai_routes = Router::new()
        .route(
            "/sdm/ijin/ajukan",
            post(handler::create_pengajuan_ijin_handler),
        )
        .route("/sdm/ijin/saya", get(handler::get_my_ijin_handler))
        .route_layer(middleware::from_fn(require_role(vec![
            "KARYAWAN".to_string(),
        ])));
    // Catatan: Rute ini akan dilindungi oleh auth_middleware utama
    // dan hanya bisa diakses oleh pegawai yang login.

    // Rute untuk admin SDM (mengelola pengajuan)
    let admin_routes = Router::new()
        .route(
            "/sdm/ijin/semua", // Endpoint untuk admin melihat semua
            get(handler::get_all_ijin_handler),
        )
        .route("/sdm/ijin/{id}/setujui", put(handler::approve_ijin_handler))
        .route("/sdm/ijin/{id}/tolak", put(handler::reject_ijin_handler))
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BASDM".to_string(), // Hanya Admin SDM & Super Admin
        ])));

    Router::new().merge(pegawai_routes).merge(admin_routes)
}
