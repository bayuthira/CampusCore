use crate::{auth::require_role, db::DbPool, handlers};
use axum::{handler::Handler, middleware, routing::{get}, Router};

pub fn matakuliah_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/api/matakuliah",
            get(handlers::matakuliah_handler::get_all_matakuliah_handler).post(
                handlers::matakuliah_handler::create_matakuliah_handler.layer(
                    middleware::from_fn(require_role(vec![
                        "SUPER_ADMIN".to_string(),
                        "KAPRODI".to_string(),
                    ])),
                ),
            ),
        )
        .route(
            "/api/matakuliah/{id}",
            get(handlers::matakuliah_handler::get_matakuliah_by_id_handler)
                .put(handlers::matakuliah_handler::update_matakuliah_handler)
                .delete(handlers::matakuliah_handler::delete_matakuliah_handler)
                .layer(middleware::from_fn(require_role(vec![
                    "SUPER_ADMIN".to_string(),
                    "KAPRODI".to_string(),
                ]))),
        )
}