use crate::{modules::auth::middleware::TokenClaims, db::DbPool, modules::general::model::SuccessResponse};
use axum::{extract::{State, Json,Query,Path}, http::StatusCode, response::IntoResponse, Extension};
use uuid::Uuid;
use super::{booking_model::{CreateBookingPayload,BookingFilter,ApprovalPayload,StartTripPayload,EndTripPayload}, booking_repo as repo};

pub async fn create_booking_handler(State(pool): State<DbPool>, Extension(claims): Extension<TokenClaims>, Json(payload): Json<CreateBookingPayload>) -> impl IntoResponse {
    let user_pemesan_id = claims.sub;
    match repo::create_booking_repo(&pool, user_pemesan_id, payload).await {
        Ok(_) => (StatusCode::CREATED, Json(SuccessResponse { message: "Pengajuan booking berhasil dibuat.".to_string() })).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_all_bookings_handler(State(pool): State<DbPool>, Query(filter): Query<BookingFilter>) -> impl IntoResponse {
    match repo::get_all_bookings_repo(&pool, filter).await {
        Ok(list) => Json(list).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn approve_booking_handler(State(pool): State<DbPool>, Extension(claims): Extension<TokenClaims>, Path(id): Path<Uuid>, Json(payload): Json<ApprovalPayload>) -> impl IntoResponse {
    let user_approve_id = claims.sub;
    match repo::approve_booking_repo(&pool, id, user_approve_id, payload.catatan).await {
        Ok(_) => (StatusCode::OK, Json(SuccessResponse { message: "Booking berhasil disetujui.".to_string() })).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn reject_booking_handler(State(pool): State<DbPool>, Extension(claims): Extension<TokenClaims>, Path(id): Path<Uuid>, Json(payload): Json<ApprovalPayload>) -> impl IntoResponse {
    let user_approve_id = claims.sub;
    match repo::reject_booking_repo(&pool, id, user_approve_id, payload.catatan).await {
        Ok(_) => (StatusCode::OK, Json(SuccessResponse { message: "Booking berhasil ditolak.".to_string() })).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn start_trip_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>, Json(payload): Json<StartTripPayload>) -> impl IntoResponse {
    match repo::start_trip_repo(&pool, id, payload).await {
        Ok(_) => (StatusCode::OK, Json(SuccessResponse { message: "Perjalanan berhasil dimulai.".to_string() })).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn end_trip_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>, Json(payload): Json<EndTripPayload>) -> impl IntoResponse {
    match repo::end_trip_repo(&pool, id, payload).await {
        Ok(_) => (StatusCode::OK, Json(SuccessResponse { message: "Perjalanan berhasil diakhiri.".to_string() })).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_my_bookings_handler(State(pool): State<DbPool>, Extension(claims): Extension<TokenClaims>) -> impl IntoResponse {
    let user_pemesan_id = claims.sub;
    match repo::get_my_bookings_repo(&pool, user_pemesan_id).await {
        Ok(list) => Json(list).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_log_by_booking_id_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match repo::get_log_by_booking_id_repo(&pool, id).await {
        Ok(log) => Json(log).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_booking_summary_handler(State(pool): State<DbPool>) -> impl IntoResponse {
    match repo::get_booking_summary_repo(&pool).await {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_bookings_by_kendaraan_id_handler(
    State(pool): State<DbPool>,
    Path(kendaraan_id): Path<Uuid>,
    Query(filter): Query<BookingFilter>,
) -> impl IntoResponse {
    match repo::get_bookings_by_kendaraan_id_repo(&pool, kendaraan_id, filter).await {
        Ok(list) => Json(list).into_response(),
        Err(e) => e.into_response(),
    }
}
