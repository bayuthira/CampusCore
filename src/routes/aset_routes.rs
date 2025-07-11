use crate::{auth::require_role, db::DbPool, handlers};
use axum::{middleware, routing::{delete, get, post, put}, Router};

pub fn aset_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/api/aset/ruangan",
            get(handlers::ruangan_handler::get_all_ruangan_handler)
            .post(handlers::ruangan_handler::create_ruangan_handler)
        )
        .route(
            "/api/aset/ruangan/{id}",
            get(handlers::ruangan_handler::get_ruangan_by_id_handler)
            .put(handlers::ruangan_handler::update_ruangan_handler)
            .delete(handlers::ruangan_handler::delete_ruangan_handler)
        )
        // Hanya SUPER_ADMIN dan STAF_AKADEMIK yang bisa mengelola aset ruangan
        .route_layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string(), "STAF_BAUM".to_string()])))
}