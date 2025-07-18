use super::{
    model::{AsetHabisPakai, AsetHabisPakaiPayload,StokTransaksiPayload},
    repo, // kita akan panggil repo::...
};
use crate::modules::auth::middleware::TokenClaims; 
use axum::Extension; 
use crate::{db::DbPool, errors::AppError};
use axum::{extract::{Path, State, Json}, http::StatusCode};
use uuid::Uuid;

pub async fn create_handler(State(pool): State<DbPool>, Json(payload): Json<AsetHabisPakaiPayload>) -> Result<(StatusCode, Json<AsetHabisPakai>), AppError> {
    let item = repo::create_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn get_all_handler(State(pool): State<DbPool>) -> Result<Json<Vec<AsetHabisPakai>>, AppError> {
    let list = repo::get_all_repo(&pool).await?;
    Ok(Json(list))
}

pub async fn get_by_id_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> Result<Json<AsetHabisPakai>, AppError> {
    let item = repo::get_by_id_repo(&pool, id).await?;
    Ok(Json(item))
}

pub async fn update_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>, Json(payload): Json<AsetHabisPakaiPayload>) -> Result<Json<AsetHabisPakai>, AppError> {
    let updated = repo::update_repo(&pool, id, payload).await?;
    Ok(Json(updated))
}

pub async fn delete_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    repo::delete_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}


pub async fn tambah_stok_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<StokTransaksiPayload>,
) -> Result<Json<AsetHabisPakai>, AppError> {
    let user_id = claims.sub;
    let updated_item = repo::tambah_stok_repo(&pool, id, payload, user_id).await?;
    Ok(Json(updated_item))
}

pub async fn ambil_stok_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<StokTransaksiPayload>,
) -> Result<Json<AsetHabisPakai>, AppError> {
    let user_id = claims.sub;
    let updated_item = repo::ambil_stok_repo(&pool, id, payload, user_id).await?;
    Ok(Json(updated_item))
}