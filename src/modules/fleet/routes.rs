use super::{booking_handler, kendaraan_handler as handler};
use crate::{db::DbPool, modules::auth::middleware::require_role};
use axum::{
    Router, middleware,
    routing::{get, post, put},
};

pub fn fleet_router() -> Router<DbPool> {
    // Rute untuk admin (Create, Update, Delete)
    let admin_routes = Router::new()
        .route("/fleet/kendaraan", post(handler::create_handler))
        .route(
            "/fleet/kendaraan/{id}",
            put(handler::update_handler).delete(handler::delete_handler),
        )
        .route(
            "/fleet/bookings",
            get(booking_handler::get_all_bookings_handler),
        )
        .route(
            "/fleet/bookings/{id}/approve",
            put(booking_handler::approve_booking_handler),
        )
        .route(
            "/fleet/bookings/{id}/reject",
            put(booking_handler::reject_booking_handler),
        )
        .route(
            "/fleet/bookings/{id}/start-trip",
            post(booking_handler::start_trip_handler),
        )
        .route(
            "/fleet/bookings/{id}/end-trip",
            post(booking_handler::end_trip_handler),
        )
        .route(
            "/fleet/bookings/{id}/log",
            get(booking_handler::get_log_by_booking_id_handler),
        )
        .route(
            "/fleet/bookings/summary",
            get(booking_handler::get_booking_summary_handler),
        )
        .route(
            "/fleet/kendaraan/{id}/summary",
            get(handler::get_vehicle_summary_handler),
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
        )
        .route(
            "/fleet/bookings",
            post(booking_handler::create_booking_handler),
        )
        .route(
            "/fleet/my-bookings",
            get(booking_handler::get_my_bookings_handler),
        );

    // Gabungkan kedua grup
    Router::new().merge(admin_routes).merge(all_user_routes)
}
