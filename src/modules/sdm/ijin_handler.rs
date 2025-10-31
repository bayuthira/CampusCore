// src/modules/sdm/ijin_handler.rs
use super::{
    ijin_model::{ApprovalIjinPayload, CreatePengajuanIjinPayload, PengajuanIjin},
    ijin_repo as repo,
    repo as pegawai_repo, // Import pegawai repo untuk get ID
};
use crate::{
    modules::auth::middleware::TokenClaims, db::DbPool, errors::AppError,
};
use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    Extension,
};
use uuid::Uuid;

/// Handler Pegawai: Mengajukan ijin baru
pub async fn create_pengajuan_ijin_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<CreatePengajuanIjinPayload>,
) -> Result<(StatusCode, Json<PengajuanIjin>), AppError> {
    // Ambil user_id dari token
    let user_id = claims.sub;
    // Dapatkan pegawai_id dari user_id
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;

    let pengajuan = repo::create_pengajuan_ijin_repo(&pool, pegawai_id, payload).await?;
    Ok((StatusCode::CREATED, Json(pengajuan)))
}

/// Handler Atasan/Admin: Menyetujui pengajuan ijin
pub async fn approve_ijin_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApprovalIjinPayload>,
) -> Result<Json<PengajuanIjin>, AppError> {
    let user_approve_id = claims.sub;
    let pengajuan = repo::approve_ijin_repo(&pool, id, user_approve_id, payload).await?;
    Ok(Json(pengajuan))
}

/// Handler Atasan/Admin: Menolak pengajuan ijin
pub async fn reject_ijin_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApprovalIjinPayload>,
) -> Result<Json<PengajuanIjin>, AppError> {
    let user_approve_id = claims.sub;
    let pengajuan = repo::reject_ijin_repo(&pool, id, user_approve_id, payload).await?;
    Ok(Json(pengajuan))
}

/// Handler Pegawai: Melihat riwayat ijin milik sendiri
pub async fn get_my_ijin_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
) -> Result<Json<Vec<PengajuanIjin>>, AppError> {
    let user_id = claims.sub;
    // Dapatkan pegawai_id dari user_id
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;

    let list = repo::get_my_ijin_repo(&pool, pegawai_id).await?;
    Ok(Json(list))
}

/// Handler Atasan/Admin: Melihat semua pengajuan ijin
pub async fn get_all_ijin_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<PengajuanIjin>>, AppError> {
    let list = repo::get_all_ijin_repo(&pool).await?;
    Ok(Json(list))
}