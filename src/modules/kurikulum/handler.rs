use crate::{db::DbPool, 
    errors::AppError, 
    modules::matakuliah::model::MataKuliahDetail};

use super::{
    model::{CreateKurikulumPayload, KurikulumDetail, UpdateKurikulumPayload,AddMataKuliahToKurikulumPayload},
    repo as kurikulum_repo,
};
use axum::{extract::{Path, State, Json}, http::StatusCode};
use uuid::Uuid;

pub async fn create_kurikulum_handler(State(pool): State<DbPool>, Json(payload): Json<CreateKurikulumPayload>) -> Result<(StatusCode, Json<KurikulumDetail>), AppError> {
    let new_kurikulum = kurikulum_repo::create_kurikulum_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(new_kurikulum)))
}

pub async fn get_all_kurikulum_handler(State(pool): State<DbPool>) -> Result<Json<Vec<KurikulumDetail>>, AppError> {
    let kurikulum_list = kurikulum_repo::get_all_kurikulum_repo(&pool).await?;
    Ok(Json(kurikulum_list))
}

pub async fn get_kurikulum_by_id_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> Result<Json<KurikulumDetail>, AppError> {
    let kurikulum = kurikulum_repo::get_kurikulum_by_id_repo_inner(&pool, id).await?;
    Ok(Json(kurikulum))
}

pub async fn update_kurikulum_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>, Json(payload): Json<UpdateKurikulumPayload>) -> Result<Json<KurikulumDetail>, AppError> {
    let updated_kurikulum = kurikulum_repo::update_kurikulum_repo(&pool, id, payload).await?;
    Ok(Json(updated_kurikulum))
}

pub async fn delete_kurikulum_handler(State(pool): State<DbPool>, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    kurikulum_repo::delete_kurikulum_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_matakuliah_to_kurikulum_handler(
    State(pool): State<DbPool>,
    Path(kurikulum_id): Path<Uuid>,
    Json(payload): Json<AddMataKuliahToKurikulumPayload>,
) -> Result<StatusCode, AppError> {
    kurikulum_repo::add_matakuliah_to_kurikulum_repo(&pool, kurikulum_id, payload).await?;
    Ok(StatusCode::CREATED)
}

pub async fn get_matakuliah_in_kurikulum_handler(
    State(pool): State<DbPool>,
    Path(kurikulum_id): Path<Uuid>,
) -> Result<Json<Vec<MataKuliahDetail>>, AppError> {
    let mk_list = kurikulum_repo::get_matakuliah_in_kurikulum_repo(&pool, kurikulum_id).await?;
    Ok(Json(mk_list))
}

pub async fn remove_matakuliah_from_kurikulum_handler(
    State(pool): State<DbPool>,
    Path((kurikulum_id, matakuliah_id)): Path<(Uuid, Uuid)>, // Ekstrak 2 param dari path
) -> Result<StatusCode, AppError> {
    kurikulum_repo::remove_matakuliah_from_kurikulum_repo(&pool, kurikulum_id, matakuliah_id).await?;
    Ok(StatusCode::NO_CONTENT)
}