// src/modules/dosen/routes.rs

use super::handler; // <-- Impor yang benar
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn dosen_router() -> Router<DbPool> {
    // Rute yang bisa diakses semua user terotentikasi
    let all_user_routes = Router::new()
        .route("/dosen", get(handler::get_all_dosen_handler))
        .route("/dosen/{id}", get(handler::get_dosen_by_id_handler));
    
    // Rute yang hanya bisa diakses oleh SUPER_ADMIN
    let admin_routes = Router::new()
        .route("/dosen", post(handler::create_dosen_handler))
        .route(
            "/dosen/{id}",
            put(handler::update_dosen_handler)
                .delete(handler::delete_dosen_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()])));
    
    // Gabungkan kedua grup router
    Router::new().merge(admin_routes).merge(all_user_routes)
}