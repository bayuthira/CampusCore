use super::handler;
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

pub fn pembelajaran_router() -> Router<DbPool> {
    let dosen_routes = Router::new()
        .route(
            "/pembelajaran/kelas-saya",
            get(handler::get_kelas_saya_handler),
        )
        .route(
            "/pembelajaran/jadwal/{jadwal_id}/pertemuan",
            get(handler::get_pertemuan_handler).post(handler::create_pertemuan_handler),
        )
        .route(
            "/pembelajaran/pertemuan/{pertemuan_id}",
            get(handler::get_detail_pertemuan_handler).put(handler::update_bap_handler),
        )
        .route(
            "/pembelajaran/pertemuan/{pertemuan_id}/buka",
            post(handler::buka_pertemuan_handler),
        )
        .route(
            "/pembelajaran/pertemuan/{pertemuan_id}/tutup",
            post(handler::tutup_pertemuan_handler),
        )
        .route(
            "/pembelajaran/pertemuan/{pertemuan_id}/presensi/{enrollment_id}",
            put(handler::manual_presensi_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec!["DOSEN".to_string()])));

    let mahasiswa_routes = Router::new()
        .route(
            "/pembelajaran/presensi/check-in",
            post(handler::check_in_mahasiswa_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "MAHASISWA".to_string()
        ])));

    Router::new().merge(dosen_routes).merge(mahasiswa_routes)
}
