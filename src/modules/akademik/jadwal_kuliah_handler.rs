// src/modules/akademik/jadwal_kuliah_handler.rs
use super::{jadwal_kuliah_model::{CreateJadwalKuliahPayload,PlotJadwalRuanganPayload,JadwalKuliahFilter,JadwalKuliahDetail} , jadwal_kuliah_repo};
use crate::{db::DbPool, errors::AppError, modules::general::model::SuccessResponse};
use axum::{extract::{State, Json,Query}, http::StatusCode};
use crate::modules::auth::middleware::TokenClaims;
use axum::Extension;

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