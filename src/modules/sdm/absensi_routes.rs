// src/modules/sdm/absensi_routes.rs
use super::absensi_handler as handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub fn absensi_router() -> Router<DbPool> {
    // Rute untuk pegawai (clock-in/out dan melihat data sendiri)
    let pegawai_routes = Router::new()
        .route("/sdm/absensi/clock-in", post(handler::clock_in_handler))
        .route("/sdm/absensi/clock-out", post(handler::clock_out_handler))
        .route("/sdm/absensi/rekap-saya", get(handler::get_my_rekap_absensi_handler))
        .route("/sdm/absensi/log-saya", get(handler::get_my_logs_for_day_handler))
                .route_layer(middleware::from_fn(require_role(vec![
            "KARYAWAN".to_string(),
        ])));
    // Rute ini akan dilindungi oleh auth_middleware utama

    // Rute untuk admin SDM (mengelola rekap)
    let admin_routes = Router::new()
        .route("/sdm/absensi/rekap-semua", get(handler::get_all_rekap_absensi_handler))
        .route("/sdm/absensi/rekap-manual", post(handler::create_rekap_manual_handler))
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BASDM".to_string(),
        ])));

    Router::new().merge(pegawai_routes).merge(admin_routes)
}