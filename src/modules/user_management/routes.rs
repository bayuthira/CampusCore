// src/routes/user_management_routes.rs

use crate::{
    auth::middleware::require_role, // Path middleware tetap karena di modul berbeda
    db::DbPool,
    modules::user_management::{handler, role_handler}, // Gunakan `super` atau path lengkap
};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn user_management_router() -> Router<DbPool> {
    Router::new()
        // Rute untuk koleksi user
        .route(
            "/users",
            get(handler::list_users_handler)
                .post(handler::create_user_handler),
        )
        // Rute untuk satu user spesifik
        .route(
            "/users/{id}", // <-- SINTAKS BENAR
            get(handler::get_user_by_id_handler)
                .put(handler::update_user_handler)
                .delete(handler::delete_user_handler),
        )
        // Rute untuk manajemen peran
        .route(
            "/users/assign-role",
            post(handler::assign_role_handler),
        )
        .route(
            "/users/revoke-role",
            delete(handler::revoke_role_handler),
        )
        .route(
            "/users/{id}/reset-password",
            put(handler::reset_password_handler),
        )
        // Rute untuk melihat daftar semua peran
        .route(
            "/roles",
             get(role_handler::get_all_roles_handler)
        )
        // Terapkan middleware untuk semua rute di atas
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
        ])))
}