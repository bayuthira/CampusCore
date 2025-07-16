// src/routes/role_routes.rs
use super::role_handler;
use crate::{
    modules::auth::middleware::require_role,
    db::DbPool,
};
use axum::{middleware, routing::get, Router}; // <-- Tambahkan middleware

pub fn role_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/roles", // Prefix /api akan ditambahkan di router utama
            get(role_handler::get_all_roles_handler),
        )
        // Terapkan middleware untuk semua rute di atas
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
        ])))
}