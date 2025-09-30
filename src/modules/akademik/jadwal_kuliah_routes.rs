// src/modules/akademik/jadwal_kuliah_routes.rs
use super::jadwal_kuliah_handler as handler;
use crate::{db::DbPool,modules::auth::middleware::require_role};
use axum::{middleware,Router, routing::{get,put,post}};


pub fn jadwal_kuliah_router() -> Router<DbPool> {
    // Grup rute yang bisa diakses oleh banyak peran (termasuk STAF_BAUM)
    let read_routes = Router::new()
        .route(
            "/akademik/jadwal-kuliah",
            get(handler::get_all_jadwal_kuliah_handler),
        )
        // Tambahkan layer untuk peran yang boleh melihat data
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_AKADEMIK".to_string(),
            "STAF_BAUM".to_string(),
            "DOSEN".to_string(), 
        ])));

    // Grup rute yang hanya untuk admin akademik
    let write_routes = Router::new()
        .route(
            "/akademik/jadwal-kuliah",
            post(handler::create_jadwal_kuliah_handler),
        )
        .route(
            "/akademik/jadwal-kuliah/{id}",
            put(handler::update_jadwal_kuliah_handler)
                .delete(handler::delete_jadwal_kuliah_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_AKADEMIK".to_string(),
        ])));

    // Gabungkan kedua grup
    Router::new().merge(read_routes).merge(write_routes)
}
