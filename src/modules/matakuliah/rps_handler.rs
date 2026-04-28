// src/modules/matakuliah/rps_handler.rs
use super::{
    rps_model::{
        RpsHeaderDetail, RpsMingguanDetail, UpsertRpsHeaderPayload, UpsertRpsMingguanPayload,
    },
    rps_repo,
};
use crate::{db::DbPool, errors::AppError};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use uuid::Uuid;

// --- HANDLER HEADER RPS ---

pub async fn get_rps_header_handler(
    State(pool): State<DbPool>,
    Path(mata_kuliah_id): Path<Uuid>,
) -> Result<Json<Option<RpsHeaderDetail>>, AppError> {
    let header = rps_repo::get_rps_header_repo(&pool, mata_kuliah_id).await?;
    Ok(Json(header))
}

pub async fn upsert_rps_header_handler(
    State(pool): State<DbPool>,
    Path(mata_kuliah_id): Path<Uuid>,
    Json(payload): Json<UpsertRpsHeaderPayload>,
) -> Result<Json<RpsHeaderDetail>, AppError> {
    let header = rps_repo::upsert_rps_header_repo(&pool, mata_kuliah_id, payload).await?;
    Ok(Json(header))
}

// --- HANDLER MINGGUAN RPS ---

pub async fn get_rps_mingguan_handler(
    State(pool): State<DbPool>,
    Path(mata_kuliah_id): Path<Uuid>,
) -> Result<Json<Vec<RpsMingguanDetail>>, AppError> {
    let list = rps_repo::get_rps_mingguan_repo(&pool, mata_kuliah_id).await?;
    Ok(Json(list))
}

pub async fn upsert_rps_mingguan_handler(
    State(pool): State<DbPool>,
    Path(mata_kuliah_id): Path<Uuid>,
    Json(payload): Json<UpsertRpsMingguanPayload>,
) -> Result<Json<RpsMingguanDetail>, AppError> {
    let mingguan = rps_repo::upsert_rps_mingguan_repo(&pool, mata_kuliah_id, payload).await?;
    Ok(Json(mingguan))
}

pub async fn delete_rps_mingguan_handler(
    State(pool): State<DbPool>,
    Path(id_mingguan): Path<Uuid>, // Perhatikan ini ID dari entri mingguan, bukan matakuliah
) -> Result<StatusCode, AppError> {
    rps_repo::delete_rps_mingguan_repo(&pool, id_mingguan).await?;
    Ok(StatusCode::NO_CONTENT)
}
