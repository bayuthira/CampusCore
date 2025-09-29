// src/modules/akademik/jadwal_kuliah_handler.rs
use super::{jadwal_kuliah_model::{CreateJadwalKuliahPayload,PlotJadwalRuanganPayload,JadwalKuliahFilter,JadwalKuliahDetail,UpdateJadwalKuliahPayload} , jadwal_kuliah_repo};
use crate::{db::DbPool, errors::AppError, modules::general::model::SuccessResponse};
use axum::{extract::{State, Json,Query,Path}, http::StatusCode};
use crate::modules::auth::middleware::TokenClaims;
use axum::Extension;
use uuid::Uuid;

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


pub async fn plot_jadwal_ruangan_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<PlotJadwalRuanganPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    let user_pembuat_id = claims.sub;
    jadwal_kuliah_repo::plot_jadwal_ruangan_repo(&pool, user_pembuat_id, payload).await?;
    Ok(Json(SuccessResponse {
        message: "Jadwal kuliah berhasil di-plot ke ruangan untuk satu semester.".to_string(),
    }))
}

pub async fn get_all_jadwal_kuliah_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<JadwalKuliahFilter>,
) -> Result<Json<Vec<JadwalKuliahDetail>>, AppError> {
    let jadwal_list = jadwal_kuliah_repo::get_all_jadwal_kuliah_repo(&pool, filter).await?;
    Ok(Json(jadwal_list))
}


pub async fn update_jadwal_kuliah_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateJadwalKuliahPayload>,
) -> Result<Json<SuccessResponse>, AppError> {
    jadwal_kuliah_repo::update_jadwal_kuliah_repo(&pool, id, payload).await?;
    Ok(Json(SuccessResponse {
        message: format!("Jadwal kuliah dengan ID {} berhasil diperbarui.", id),
    }))
}

pub async fn delete_jadwal_kuliah_handler(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    jadwal_kuliah_repo::delete_jadwal_kuliah_repo(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}