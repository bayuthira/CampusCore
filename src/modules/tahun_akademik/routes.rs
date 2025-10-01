use super::handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{middleware, routing::{get,post,put}, Router};

pub fn tahun_akademik_router() -> Router<DbPool> {
    // Grup rute untuk operasi tulis (Create, Update, Delete)
    // Hanya untuk SUPER_ADMIN
    let write_routes = Router::new()
        .route("/tahun-akademik", post(handler::create_tahun_akademik_handler))
        .route(
            "/tahun-akademik/{id}",
            put(handler::update_tahun_akademik_handler)
                .delete(handler::delete_tahun_akademik_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()])));

    // Grup rute untuk operasi baca (Read)
    // Bisa diakses oleh lebih banyak peran
    let read_routes = Router::new()
        .route("/tahun-akademik", get(handler::get_all_tahun_akademik_handler))
        .route(
            "/tahun-akademik/{id}",
            get(handler::get_tahun_akademik_by_id_handler),
        )
        // Layer ini bisa Anda sesuaikan. Contoh: semua user terotentikasi boleh melihat.
        // Jika tidak ada .route_layer(), maka hanya perlu login.
        // Jika ingin membatasi, tambahkan .route_layer() seperti di bawah.
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_AKADEMIK".to_string(),
            "STAF_BAUM".to_string(),
            "DOSEN".to_string(),
            "MAHASISWA".to_string(),
        ])));
    
    // Gabungkan kedua grup
    Router::new().merge(write_routes).merge(read_routes)
}