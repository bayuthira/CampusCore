use crate::{
    modules::auth::middleware::TokenClaims,
    db::DbPool,
    errors::AppError,
    modules::aset::{
        jadwal_ruangan_model::{CreateJadwalPayload, JadwalRuangan},
        jadwal_ruangan_repo,
    },
};
use axum::{
    extract::{State, Json},
    http::StatusCode,
    Extension,
};

pub async fn create_jadwal_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<CreateJadwalPayload>,
) -> Result<(StatusCode, Json<Vec<JadwalRuangan>>), AppError> {
    let user_pembuat_id = claims.sub;
    let new_jadwals =
        jadwal_ruangan_repo::create_jadwal_repo(&pool, user_pembuat_id, payload).await?;
    Ok((StatusCode::CREATED, Json(new_jadwals)))
}