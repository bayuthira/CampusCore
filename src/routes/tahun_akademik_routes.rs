use crate::{auth::require_role, db::DbPool, handlers};
use axum::{middleware, routing::{get}, Router};

pub fn tahun_akademik_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/api/tahun-akademik",
            get(handlers::tahun_akademik_handler::get_all_tahun_akademik_handler)
                .post(handlers::tahun_akademik_handler::create_tahun_akademik_handler),
        )
        .route(
            "/api/tahun-akademik/{id}",
            get(handlers::tahun_akademik_handler::get_tahun_akademik_by_id_handler)
                .put(handlers::tahun_akademik_handler::update_tahun_akademik_handler)
                .delete(handlers::tahun_akademik_handler::delete_tahun_akademik_handler),
        )
        .layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()])))
}