// src/handlers/tahun_akademik_handler.rs

use super::{
    model::{TaPayload, TahunAkademik},
    repo as tahun_akademik_repo,
};

use crate::{
    db::DbPool,
    errors::AppError,
};
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
};
use uuid::Uuid;

pub async fn create_tahun_akademik_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<TaPayload>,
) -> Result<(StatusCode, Json<TahunAkademik>), AppError> {
    let ta = tahun_akademik_repo::create_tahun_akademik_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(ta)))
}

pub async fn get_all_tahun_akademik_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<TahunAkademik>>, AppError> {
    let ta_list = tahun_akademik_repo::get_all_tahun_akademik_repo(&pool).await?;
    Ok(Json(ta_list))
}

pub async fn get_tahun_akademik_by_id_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<TahunAkademik>, AppError> {
    let ta = tahun_akademik_repo::get_tahun_akademik_by_id_repo(&pool, id).await?;
    Ok(Json(ta))
}

pub async fn update_tahun_akademik_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TaPayload>,
) -> Result<Json<TahunAkademik>, AppError> {
    let updated_ta = tahun_akademik_repo::update_tahun_akademik_repo(&pool, id, payload).await?;
    Ok(Json(updated_ta))
}

pub async fn delete_tahun_akademik_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    tahun_akademik_repo::delete_tahun_akademik_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}