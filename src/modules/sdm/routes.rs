// src/modules/sdm/routes.rs

use super::{
    absensi_routes, cuti_routes, dokumen_handler, handler, ijin_routes, penempatan_routes,
    riwayat_jad_handler, riwayat_pendidikan_handler, riwayat_serdos_handler,
    riwayat_sertifikat_handler, riwayat_sk_handler, surat_tugas_routes, unit_kerja_routes,
};
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};

pub fn sdm_router() -> Router<DbPool> {
    // Grup 1: Aksi untuk STAF_BASDM & SUPER_ADMIN (Create, Read, Update)
    let sdm_staff_routes = Router::new()
        .route(
            "/sdm/pegawai",
            get(handler::get_all_pegawai_handler).post(handler::create_pegawai_handler),
        )
        .route(
            "/sdm/pegawai/{id}",
            get(handler::get_pegawai_by_id_handler).put(handler::update_pegawai_handler),
        )
        .route(
            "/sdm/pegawai/{pegawai_id}/pendidikan",
            get(riwayat_pendidikan_handler::get_all_by_pegawai_id_handler)
                .post(riwayat_pendidikan_handler::create_handler),
        )
        .route(
            "/sdm/pendidikan/{id}",
            put(riwayat_pendidikan_handler::update_handler)
                .delete(riwayat_pendidikan_handler::delete_handler),
        )
        .route(
            "/sdm/pegawai/{pegawai_id}/riwayat-sk",
            get(riwayat_sk_handler::get_all_by_pegawai_id_handler)
                .post(riwayat_sk_handler::create_handler),
        )
        .route(
            "/sdm/riwayat-sk/{id}",
            put(riwayat_sk_handler::update_handler).delete(riwayat_sk_handler::delete_handler),
        )
        .route(
            "/sdm/dokumen", // Khusus admin melihat semua dokumen di perusahaan
            get(dokumen_handler::get_all_dokumen_admin_handler),
        )
        .route(
            "/sdm/pegawai/{pegawai_id}/sertifikat",
            get(riwayat_sertifikat_handler::get_all_by_pegawai_id_handler)
                .post(riwayat_sertifikat_handler::create_handler),
        )
        .route(
            "/sdm/sertifikat/{id}",
            put(riwayat_sertifikat_handler::update_handler)
                .delete(riwayat_sertifikat_handler::delete_handler),
        )
        .route(
            "/sdm/pegawai/{pegawai_id}/jad",
            get(riwayat_jad_handler::get_all_by_pegawai_id_handler)
                .post(riwayat_jad_handler::create_handler),
        )
        .route(
            "/sdm/jad/{id}",
            put(riwayat_jad_handler::update_handler).delete(riwayat_jad_handler::delete_handler),
        )
        .route(
            "/sdm/pegawai/{pegawai_id}/serdos",
            get(riwayat_serdos_handler::get_all_by_pegawai_id_handler)
                .post(riwayat_serdos_handler::create_handler),
        )
        .route(
            "/sdm/serdos/{id}",
            put(riwayat_serdos_handler::update_handler)
                .delete(riwayat_serdos_handler::delete_handler),
        )
        .route(
            "/sdm/absensi/wajah/{pegawai_id}/audit",
            put(crate::modules::sdm::absensi_wajah_handler::audit_wajah_handler),
        )
        .route(
            "/sdm/absensi/wajah/{pegawai_id}",
            delete(crate::modules::sdm::absensi_wajah_handler::delete_wajah_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BASDM".to_string(),
        ])));

    // Grup 2: Aksi Khusus Dokumen Bersama (Bisa diakses Admin & Karyawan)
    let shared_dokumen_routes = Router::new()
        .route(
            "/sdm/absensi/wajah/{pegawai_id}",
            get(crate::modules::sdm::absensi_wajah_handler::get_foto_wajah_handler),
        )
        .route(
            "/sdm/{entity_type}/{entity_id}/dokumen",
            post(dokumen_handler::upload_dokumen_handler)
                .get(dokumen_handler::get_all_dokumen_handler),
        )
        .route(
            "/sdm/dokumen/{id}",
            delete(dokumen_handler::delete_dokumen_handler),
        )
        .route(
            "/sdm/absensi/enroll-wajah",
            post(crate::modules::sdm::absensi_wajah_handler::enroll_wajah_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BASDM".to_string(),
            "KARYAWAN".to_string(), // Karyawan diizinkan, perlindungan berlapis ada di handler
        ])));

    // Grup 3: Aksi khusus SUPER_ADMIN (Delete, Create User)
    let super_admin_routes = Router::new()
        .route("/sdm/pegawai/{id}", delete(handler::delete_pegawai_handler))
        .route(
            "/sdm/pegawai/{id}/create-user",
            post(handler::create_user_for_pegawai_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
        ])));

    // Gabungkan semua grup menjadi satu router untuk modul SDM
    Router::new()
        .merge(sdm_staff_routes)
        .merge(shared_dokumen_routes) // <-- Gabungkan rute dokumen bersamanya disini
        .merge(super_admin_routes)
        .merge(cuti_routes::cuti_router())
        .merge(ijin_routes::ijin_router())
        .merge(absensi_routes::absensi_router())
        .merge(unit_kerja_routes::unit_kerja_router())
        .merge(penempatan_routes::penempatan_router())
        .merge(surat_tugas_routes::surat_tugas_router())
}
