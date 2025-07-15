use crate::{
    db::DbPool,
    errors::AppError,
    models::aset_model::{AsetDetail, AsetPayload,HistoriAsetDetail},
    repositories::{aset_repo,histori_aset_repo},
};
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
};
use uuid::Uuid;

/// Handler untuk membuat Aset baru
pub async fn create_aset_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<AsetPayload>,
) -> Result<(StatusCode, Json<AsetDetail>), AppError> {
    let aset = aset_repo::create_aset_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(aset)))
}

/// Handler untuk mendapatkan semua Aset
pub async fn get_all_aset_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<AsetDetail>>, AppError> {
    let list = aset_repo::get_all_aset_repo(&pool).await?;
    Ok(Json(list))
}

/// Handler untuk mendapatkan satu Aset berdasarkan ID
pub async fn get_aset_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<AsetDetail>, AppError> {
    let aset = aset_repo::get_aset_by_id_repo(&pool, id).await?;
    Ok(Json(aset))
}

/// Handler untuk memperbarui Aset
pub async fn update_aset_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AsetPayload>,
) -> Result<Json<AsetDetail>, AppError> {
    let updated = aset_repo::update_aset_repo(&pool, id, payload).await?;
    Ok(Json(updated))
}

/// Handler untuk menghapus Aset
pub async fn delete_aset_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    aset_repo::delete_aset_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_aset_histori_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>, // ID dari aset
) -> Result<Json<Vec<HistoriAsetDetail>>, AppError> {
    let histori = histori_aset_repo::get_histori_by_aset_id_repo(&pool, id).await?;
    Ok(Json(histori))
}