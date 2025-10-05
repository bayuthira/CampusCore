use crate::{modules::auth::middleware::TokenClaims, db::DbPool, modules::fleet::{booking_model::CreateBookingPayload, booking_repo as repo}, modules::general::model::SuccessResponse};
use axum::{extract::{State, Json}, http::StatusCode, response::IntoResponse, Extension};

pub async fn create_booking_handler(State(pool): State<DbPool>, Extension(claims): Extension<TokenClaims>, Json(payload): Json<CreateBookingPayload>) -> impl IntoResponse {
    let user_pemesan_id = claims.sub;
    match repo::create_booking_repo(&pool, user_pemesan_id, payload).await {
        Ok(_) => (StatusCode::CREATED, Json(SuccessResponse { message: "Pengajuan booking berhasil dibuat.".to_string() })).into_response(),
        Err(e) => e.into_response(),
    }
}