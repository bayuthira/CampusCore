use crate::{db::DbPool, errors::AppError, modules::fleet::{kendaraan_model::{Kendaraan, KendaraanPayload}, kendaraan_repo as repo}};
use axum::{extract::{Path, State, Json}, http::StatusCode, response::IntoResponse};
use uuid::Uuid;

pub async fn create_handler(State(pool): State<DbPool>, Json(payload): Json<KendaraanPayload>) -> impl IntoResponse {
    match repo::create_repo(&pool, payload).await {
        Ok(item) => (StatusCode::CREATED, Json(item)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_all_handler(State(pool): State<DbPool>) -> impl IntoResponse {
    match repo::get_all_repo(&pool).await {
        Ok(list) => Json(list).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_by_id_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match repo::get_by_id_repo(&pool, id).await {
        Ok(item) => Json(item).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn update_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>, Json(payload): Json<KendaraanPayload>) -> impl IntoResponse {
    match repo::update_repo(&pool, id, payload).await {
        Ok(item) => Json(item).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn delete_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match repo::delete_repo(&pool, id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}