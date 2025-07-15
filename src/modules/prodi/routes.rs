// src/routes/prodi_routes.rs

use crate::{modules::auth::middleware::require_role, db::DbPool, modules::prodi::handler};
use axum::{
    handler::Handler, // <-- Perlu di-import untuk bisa menggunakan .layer()
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn prodi_router() -> Router<DbPool> {
    // Grup rute yang hanya bisa diakses oleh SUPER_ADMIN
    let admin_routes = Router::new()
        .route("/prodi", post(handler::create_prodi_handler))
        .route(
            "/prodi/{id}",
            put(handler::update_prodi_handler)
                .delete(handler::delete_prodi_handler),
        )
        // Layer ini hanya berlaku untuk rute di grup `admin_routes`
        .route_layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()])));

    // Grup rute yang bisa diakses oleh semua user yang sudah login
    let all_user_routes = Router::new()
        .route("/prodi", get(handler::get_all_prodi_handler))
        .route("/prodi/{id}", get(handler::get_prodi_by_id_handler));

    // Gabungkan kedua grup menjadi satu router untuk modul prodi
    Router::new().merge(admin_routes).merge(all_user_routes)
}