use crate::{modules::auth::middleware::TokenClaims, db::DbPool};
use axum::{extract::{Path, State, Json, Query}, http::StatusCode, response::IntoResponse, Extension};
use super:: {servis_model::{ServisPayload,ServisFilter}, servis_repo as repo};
use uuid::Uuid;

pub async fn create_servis_handler(State(pool): State<DbPool>, Extension(claims): Extension<TokenClaims>, Path(kendaraan_id): Path<Uuid>, Json(payload): Json<ServisPayload>) -> impl IntoResponse {
    let user_pencatat_id = claims.sub;
    match repo::create_servis_repo(&pool, kendaraan_id, user_pencatat_id, payload).await {
        Ok(item) => (StatusCode::CREATED, Json(item)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_all_servis_by_kendaraan_id_handler(
    State(pool): State<DbPool>,
    Path(kendaraan_id): Path<Uuid>,
    Query(filter): Query<ServisFilter>,
) -> impl IntoResponse {
    match repo::get_all_servis_by_kendaraan_id_repo(&pool, kendaraan_id, filter).await {
        Ok(list) => Json(list).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_servis_by_id_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match repo::get_servis_by_id_repo(&pool, id).await {
        Ok(item) => Json(item).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn update_servis_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>, Json(payload): Json<ServisPayload>) -> impl IntoResponse {
    match repo::update_servis_repo(&pool, id, payload).await {
        Ok(item) => Json(item).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn delete_servis_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match repo::delete_servis_repo(&pool, id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => e.into_response(),
    }
}

