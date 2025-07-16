use super::handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{handler::Handler, middleware, routing::{get}, Router};

pub fn matakuliah_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/matakuliah",
            get(handler::get_all_matakuliah_handler).post(
                handler::create_matakuliah_handler.layer(
                    middleware::from_fn(require_role(vec![
                        "SUPER_ADMIN".to_string(),
                        "KAPRODI".to_string(),
                    ])),
                ),
            ),
        )
        .route(
            "/matakuliah/{id}",
            get(handler::get_matakuliah_by_id_handler)
                .put(handler::update_matakuliah_handler)
                .delete(handler::delete_matakuliah_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "KAPRODI".to_string(),
                ]))),
        )
}