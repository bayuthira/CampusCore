// src/routes/user_management_routes.rs
use crate::{auth::require_role, db::DbPool, handlers};
use axum::{middleware, routing::{get, post, delete}, Router};

pub fn user_management_router() -> Router<DbPool> {
    Router::new()
        .route("/api/users", get(handlers::user_management_handler::list_users_handler))
        .route("/api/users/assign-role", post(handlers::user_management_handler::assign_role_handler))
        .route("/api/users/revoke-role", delete(handlers::user_management_handler::revoke_role_handler))
        // Terapkan middleware untuk semua rute di atas
        .layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()])))
}