use super::{kendaraan_handler as handler,booking_handler};
use crate::{modules::auth::middleware::require_role, db::DbPool};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

pub fn fleet_router() -> Router<DbPool> {
    // Rute untuk admin (Create, Update, Delete)
    let admin_routes = Router::new()
        .route(
            "/fleet/kendaraan",
            post(handler::create_handler),
        )
        .route(
            "/fleet/kendaraan/{id}",
            put(handler::update_handler).delete(handler::delete_handler),
        )
        .route_layer(middleware::from_fn(require_role(vec![
            "SUPER_ADMIN".to_string(),
            "STAF_BAUM".to_string(),
        ])));

    // Rute yang bisa diakses semua user terotentikasi (Read)
    let all_user_routes = Router::new()
        .route("/fleet/kendaraan", get(handler::get_all_handler))
        .route("/fleet/kendaraan/{id}", get(handler::get_by_id_handler))
        .route(
            "/fleet/kendaraan-tersedia",
            get(handler::search_available_vehicles_handler),
        ).route("/fleet/bookings", post(booking_handler::create_booking_handler));
    
    // Gabungkan kedua grup
    Router::new().merge(admin_routes).merge(all_user_routes)
}