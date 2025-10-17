use super::{
    model::{RiwayatPendidikan, RiwayatPendidikanPayload},
    riwayat_pendidikan_repo as repo,
};
use crate::{db::DbPool, errors::AppError};
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
};
use uuid::Uuid;

pub async fn create_handler(State(pool): State<DbPool>, Path(pegawai_id): Path<Uuid>, Json(payload): Json<RiwayatPendidikanPayload>) -> Result<(StatusCode, Json<RiwayatPendidikan>), AppError> {
    let item = repo::create_riwayat_pendidikan_repo(&pool, pegawai_id, payload).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn get_all_by_pegawai_id_handler(State(pool): State<DbPool>, Path(pegawai_id): Path<Uuid>) -> Result<Json<Vec<RiwayatPendidikan>>, AppError> {
    let list = repo::get_all_riwayat_pendidikan_by_pegawai_id_repo(&pool, pegawai_id).await?;
    Ok(Json(list))
}

pub async fn update_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>, Json(payload): Json<RiwayatPendidikanPayload>) -> Result<Json<RiwayatPendidikan>, AppError> {
    let item = repo::update_riwayat_pendidikan_repo(&pool, id, payload).await?;
    Ok(Json(item))
}

pub async fn delete_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    repo::delete_riwayat_pendidikan_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}