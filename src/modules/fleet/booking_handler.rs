use crate::{modules::auth::middleware::TokenClaims, db::DbPool, modules::fleet::{booking_model::{CreateBookingPayload,BookingFilter,ApprovalPayload}, booking_repo as repo}, modules::general::model::SuccessResponse};
use axum::{extract::{State, Json,Query,Path}, http::StatusCode, response::IntoResponse, Extension};
use uuid::Uuid;

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
