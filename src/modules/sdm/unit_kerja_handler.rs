// src/modules/sdm/unit_kerja_handler.rs
use super::{
    unit_kerja_model::{UnitKerja, UnitKerjaPayload},
    unit_kerja_repo as repo,
};
use crate::{db::DbPool, errors::AppError};
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
};
use uuid::Uuid;

pub async fn create_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<UnitKerjaPayload>,
) -> Result<(StatusCode, Json<UnitKerja>), AppError> {
    let item = repo::create_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn get_all_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<UnitKerja>>, AppError> {
    let list = repo::get_all_repo(&pool).await?;
    Ok(Json(list))
}

pub async fn get_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnitKerja>, AppError> {
    let item = repo::get_by_id_repo(&pool, id).await?;
    Ok(Json(item))
}

pub async fn update_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UnitKerjaPayload>,
) -> Result<Json<UnitKerja>, AppError> {
    let item = repo::update_repo(&pool, id, payload).await?;
    Ok(Json(item))
}

pub async fn delete_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    repo::delete_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}