use super::handler;
use crate::db::DbPool;
use axum::{Router, routing::get};

pub fn lookup_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/lookups/enrollment-statuses",
            get(handler::get_enrollment_statuses_handler),
        )
        .route(
            "/lookups/kondisi-aset",
            get(handler::get_kondisi_aset_handler),
        )
        .route(
            "/lookups/aset-histori-statuses",
            get(handler::get_aset_histori_statuses_handler),
        )
        .route("/lookups/users", get(handler::search_users_handler))
        .route("/lookups/tipe-biaya", get(handler::get_tipe_biaya_handler))
        .route(
            "/lookups/peran-dosen-pengampu",
            get(handler::get_peran_dosen_pengampu_handler),
        )
}
