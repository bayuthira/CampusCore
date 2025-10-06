use super::servis_handler as handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{middleware, routing::{get}, Router};

pub fn servis_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/fleet/kendaraan/{kendaraan_id}/servis",
            get(handler::get_all_servis_by_kendaraan_id_handler)
            .post(handler::create_servis_handler)
        )
        .route(
            "/fleet/servis/{id}",
            get(handler::get_servis_by_id_handler)
            .put(handler::update_servis_handler)
            .delete(handler::delete_servis_handler)
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BAUM".to_string(),
        ])))
}