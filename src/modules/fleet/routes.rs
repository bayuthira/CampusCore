use super::kendaraan_handler as handler;
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{middleware, routing::{get}, Router};

pub fn fleet_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/fleet/kendaraan",
            get(handler::get_all_handler).post(handler::create_handler),
        )
        .route(
            "/fleet/kendaraan/{id}",
            get(handler::get_by_id_handler)
                .put(handler::update_handler)
                .delete(handler::delete_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BAUM".to_string(),
        ])))
}