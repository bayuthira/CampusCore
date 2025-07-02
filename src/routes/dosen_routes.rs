use crate::{auth::require_role, db::DbPool, handlers};
use axum::{handler::Handler, middleware, routing::{get}, Router};

pub fn dosen_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/api/dosen",
            get(handlers::dosen_handler::get_all_dosen_handler).post(
                handlers::dosen_handler::create_dosen_handler
                    .layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()]))),
            ),
        )
        .route(
            "/api/dosen/{id}",
            get(handlers::dosen_handler::get_dosen_by_id_handler)
                .put(handlers::dosen_handler::update_dosen_handler)
                .delete(handlers::dosen_handler::delete_dosen_handler)
                .layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()]))),
        )
}