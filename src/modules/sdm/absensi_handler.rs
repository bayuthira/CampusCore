// src/modules/sdm/absensi_handler.rs
use super::{
    absensi_model::{
        ClockPayload, LogAbsensi, RekapAbsensiFilter, RekapAbsensiHarian, RekapManualPayload, LogDayFilter,
    },
    absensi_repo as repo,
    repo as pegawai_repo, // Untuk get_pegawai_id
};
use crate::{
    modules::auth::middleware::TokenClaims, db::DbPool, errors::AppError,
};
use axum::{
    extract::{Query, State, Json},
    http::StatusCode,
    Extension,
};

/// Handler Pegawai: Melakukan Clock In
pub async fn clock_in_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<ClockPayload>,
) -> Result<(StatusCode, Json<LogAbsensi>), AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;
    let log = repo::clock_in_repo(&pool, pegawai_id, payload).await?;
    Ok((StatusCode::CREATED, Json(log)))
}

/// Handler Pegawai: Melakukan Clock Out
pub async fn clock_out_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<ClockPayload>,
) -> Result<(StatusCode, Json<LogAbsensi>), AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;
    let log = repo::clock_out_repo(&pool, pegawai_id, payload).await?;
    Ok((StatusCode::CREATED, Json(log)))
}

/// Handler Admin: Membuat atau mengoreksi rekap absensi harian
pub async fn create_rekap_manual_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<RekapManualPayload>,
) -> Result<(StatusCode, Json<RekapAbsensiHarian>), AppError> {
    let rekap = repo::create_rekap_manual_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(rekap)))
}

/// Handler Pegawai: Melihat rekap absensi bulanan milik sendiri
pub async fn get_my_rekap_absensi_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(filter): Query<RekapAbsensiFilter>,
) -> Result<Json<Vec<RekapAbsensiHarian>>, AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;
    let list = repo::get_my_rekap_absensi_repo(&pool, pegawai_id, filter).await?;
    Ok(Json(list))
}

/// Handler Pegawai: Melihat log clock-in/out untuk satu hari
pub async fn get_my_logs_for_day_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(filter): Query<LogDayFilter>, // <-- 2. UBAH TIPE DI SINI
) -> Result<Json<Vec<LogAbsensi>>, AppError> {
    let user_id = claims.sub;
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;
    let list = repo::get_my_logs_for_day_repo(&pool, pegawai_id, filter.tanggal).await?; // <-- 3. Gunakan filter.tanggal
    Ok(Json(list))
}

/// Handler Admin: Melihat rekap absensi bulanan semua pegawai
pub async fn get_all_rekap_absensi_handler(
    State(pool): State<DbPool>,
    Query(filter): Query<RekapAbsensiFilter>,
) -> Result<Json<Vec<RekapAbsensiHarian>>, AppError> {
    let list = repo::get_all_rekap_absensi_repo(&pool, filter).await?;
    Ok(Json(list))
}