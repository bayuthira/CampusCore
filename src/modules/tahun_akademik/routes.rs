use super::handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{middleware, routing::{get}, Router};

pub fn tahun_akademik_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/tahun-akademik",
            get(handler::get_all_tahun_akademik_handler)
                .post(handler::create_tahun_akademik_handler),
        )
        .route(
            "/tahun-akademik/{id}",
            get(handler::get_tahun_akademik_by_id_handler)
                .put(handler::update_tahun_akademik_handler)
                .delete(handler::delete_tahun_akademik_handler),
        )
        .layer(middleware::from_fn(require_role(vec!["SUPER_ADMIN".to_string()])))
}