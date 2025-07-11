use crate::{db::DbPool, errors::AppError, models::ruangan_model::{Ruangan, RuanganPayload}, repositories::ruangan_repo};
use axum::{extract::{Path, State, Json}, http::StatusCode};
use uuid::Uuid;

pub async fn create_ruangan_handler(State(pool): State<DbPool>, Json(payload): Json<RuanganPayload>) -> Result<(StatusCode, Json<Ruangan>), AppError> {
    let ruangan = ruangan_repo::create_ruangan_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(ruangan)))
}

pub async fn get_all_ruangan_handler(State(pool): State<DbPool>) -> Result<Json<Vec<Ruangan>>, AppError> {
    let ruangan_list = ruangan_repo::get_all_ruangan_repo(&pool).await?;
    Ok(Json(ruangan_list))
}

pub async fn get_ruangan_by_id_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> Result<Json<Ruangan>, AppError> {
    let ruangan = ruangan_repo::get_ruangan_by_id_repo(&pool, id).await?;
    Ok(Json(ruangan))
}

pub async fn update_ruangan_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>, Json(payload): Json<RuanganPayload>) -> Result<Json<Ruangan>, AppError> {
    let updated_ruangan = ruangan_repo::update_ruangan_repo(&pool, id, payload).await?;
    Ok(Json(updated_ruangan))
}

pub async fn delete_ruangan_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    ruangan_repo::delete_ruangan_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}