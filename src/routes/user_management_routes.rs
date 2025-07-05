// src/routes/user_management_routes.rs
use crate::{auth::require_role, db::DbPool, handlers};
use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
pub fn user_management_router() -> Router<DbPool> {
    Router::new()
        // Rute untuk koleksi user
        .route(
            "/api/users",
            get(handlers::user_management_handler::list_users_handler)
            .post(handlers::user_management_handler::create_user_handler),
        )
        
        // Rute untuk satu user spesifik
        .route(
            "/api/users/{id}",
            get(handlers::user_management_handler::get_user_by_id_handler)
                .put(handlers::user_management_handler::update_user_handler)
                .delete(handlers::user_management_handler::delete_user_handler),
        )
        // Rute untuk manajemen peran
        .route(
            "/api/users/assign-role",
            post(handlers::user_management_handler::assign_role_handler),
        )
        .route(
            "/api/users/revoke-role",
            delete(handlers::user_management_handler::revoke_role_handler),
        )
        .route(
            "/api/users/{id}/reset-password",
            put(handlers::user_management_handler::reset_password_handler),
        )
        // Terapkan middleware untuk semua rute di atas
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
        ])))
}
