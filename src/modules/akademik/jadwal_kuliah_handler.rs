// src/modules/akademik/jadwal_kuliah_handler.rs
use super::{jadwal_kuliah_model::CreateJadwalKuliahPayload, jadwal_kuliah_repo};
use crate::{db::DbPool, errors::AppError, modules::general::model::SuccessResponse};
use axum::{extract::{State, Json}, http::StatusCode};

pub async fn create_jadwal_kuliah_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateJadwalKuliahPayload>,
) -> Result<(StatusCode, Json<SuccessResponse>), AppError> {
    let jadwal_id = jadwal_kuliah_repo::create_jadwal_kuliah_repo(&pool, payload).await?;
    Ok((
        StatusCode::CREATED,
        Json(SuccessResponse {
            message: format!("Jadwal kuliah berhasil dibuat dengan ID: {}", jadwal_id),
        }),
    ))
}