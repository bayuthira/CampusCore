use crate::{
    modules::auth::middleware::TokenClaims,
    db::DbPool,
    errors::AppError,
    modules::aset::{
        jadwal_ruangan_model::{CreateJadwalPayload, JadwalRuangan,JadwalRuanganFilter},
        jadwal_ruangan_repo,
    },
};
use axum::{
    extract::{State, Json,Query,Path},
    http::StatusCode,
    Extension,
};
use uuid::Uuid;

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


pub async fn get_jadwal_by_ruangan_handler(
    State(pool): State<DbPool>,
    Path(ruangan_id): Path<Uuid>,
    Query(filter): Query<JadwalRuanganFilter>,
) -> Result<Json<Vec<JadwalRuangan>>, AppError> {
    let list =
        jadwal_ruangan_repo::get_jadwal_by_ruangan_repo(&pool, ruangan_id, filter).await?;
    Ok(Json(list))
}


pub async fn delete_jadwal_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    jadwal_ruangan_repo::delete_jadwal_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
