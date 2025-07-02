use crate::{auth::require_role, db::DbPool, handlers};
use axum::{handler::Handler, middleware, routing::{get}, Router};

pub fn prodi_router() -> Router<DbPool> {
    Router::new().route(
        "/api/prodi",
        get(handlers::prodi_handler::get_all_prodi_handler).post(
            handlers::prodi_handler::create_prodi_handler
                .layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()]))),
        ),
    )
}