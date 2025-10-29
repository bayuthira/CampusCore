use super::cuti_handler as handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

pub fn cuti_router() -> Router<DbPool> {
    // Rute untuk pegawai (mengajukan dan melihat riwayat sendiri)
    let pegawai_routes = Router::new()
        .route(
            "/sdm/cuti/ajukan",
            post(handler::create_pengajuan_cuti_handler),
        )
        .route(
            "/sdm/cuti/saya",
            get(handler::get_my_cuti_handler),
        )
        .route(
            "/sdm/cuti/kuota-saya",
            get(handler::get_my_kuota_cuti_handler)
        );
        // Catatan: Rute ini akan dilindungi oleh auth_middleware utama di routes/mod.rs
        // Kita bisa tambahkan role spesifik jika perlu, misal: DOSEN, STAF_BASDM

    // Rute untuk admin SDM (mengelola pengajuan dan jatah)
    let admin_routes = Router::new()
        .route(
            "/sdm/cuti/semua", // Endpoint untuk admin melihat semua
            get(handler::get_all_cuti_handler),
        )
        .route(
            "/sdm/cuti/jatah", // Endpoint untuk generate jatah
            post(handler::create_jatah_cuti_handler),
        )
        .route(
            "/sdm/cuti/{id}/setujui",
            put(handler::approve_cuti_handler),
        )
        .route(
            "/sdm/cuti/{id}/tolak",
            put(handler::reject_cuti_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BASDM".to_string(), // Hanya Admin SDM & Super Admin
        ])));

    Router::new().merge(pegawai_routes).merge(admin_routes)
}