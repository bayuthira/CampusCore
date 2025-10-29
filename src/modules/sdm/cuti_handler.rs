// src/modules/sdm/cuti_handler.rs
use super::{
    cuti_model::{
        ApprovalCutiPayload, CreateJatahCutiPayload, CreatePengajuanCutiPayload, JatahCuti, KuotaCutiDetail,KuotaFilter,
        PengajuanCuti,
    },
    cuti_repo as repo,
    repo as pegawai_repo,
};
use crate::{modules::auth::middleware::TokenClaims, db::DbPool, errors::AppError};
use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Extension,
};
use uuid::Uuid;

/// Handler Khusus Admin: Membuat/mengatur jatah cuti tahunan
pub async fn create_jatah_cuti_handler(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateJatahCutiPayload>,
) -> Result<(StatusCode, Json<JatahCuti>), AppError> {
    let jatah = repo::create_jatah_cuti_repo(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(jatah)))
}

/// Handler Pegawai: Mengajukan cuti baru
pub async fn create_pengajuan_cuti_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<CreatePengajuanCutiPayload>,
) -> Result<(StatusCode, Json<PengajuanCuti>), AppError> {
    let user_id = claims.sub;
    
    // --- PERBAIKAN DI SINI ---
    // Panggil fungsi baru yang ringan untuk mendapatkan ID saja
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;
    
    let pengajuan = repo::create_pengajuan_cuti_repo(&pool, pegawai_id, payload).await?;
    Ok((StatusCode::CREATED, Json(pengajuan)))
}

/// Handler Atasan/Admin: Menyetujui pengajuan cuti
pub async fn approve_cuti_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApprovalCutiPayload>,
) -> Result<Json<PengajuanCuti>, AppError> {
    let user_approve_id = claims.sub;
    let pengajuan = repo::approve_cuti_repo(&pool, id, user_approve_id, payload).await?;
    Ok(Json(pengajuan))
}

/// Handler Atasan/Admin: Menolak pengajuan cuti
pub async fn reject_cuti_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApprovalCutiPayload>,
) -> Result<Json<PengajuanCuti>, AppError> {
    let user_approve_id = claims.sub;
    let pengajuan = repo::reject_cuti_repo(&pool, id, user_approve_id, payload).await?;
    Ok(Json(pengajuan))
}

/// Handler Pegawai: Melihat riwayat cuti milik sendiri
pub async fn get_my_cuti_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
) -> Result<Json<Vec<PengajuanCuti>>, AppError> {
    let user_id = claims.sub;

    // --- PERBAIKAN DI SINI JUGA ---
    // Panggil fungsi baru yang ringan
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;

    let list = repo::get_my_cuti_repo(&pool, pegawai_id).await?;
    Ok(Json(list))
}

/// Handler Atasan/Admin: Melihat semua pengajuan cuti (bisa difilter)
pub async fn get_all_cuti_handler(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<PengajuanCuti>>, AppError> {
    let list = repo::get_all_cuti_repo(&pool).await?;
    Ok(Json(list))
}

pub async fn get_my_kuota_cuti_handler(
    State(pool): State<DbPool>,
    Extension(claims): Extension<TokenClaims>,
    Query(filter): Query<KuotaFilter>, // Menerima ?tahun=...
) -> Result<Json<KuotaCutiDetail>, AppError> {
    let user_id = claims.sub;
    // Cari pegawai_id berdasarkan user_id
    let pegawai_id = pegawai_repo::get_pegawai_id_from_user_id_repo(&pool, user_id).await?;
    
    let kuota = repo::get_kuota_cuti_repo(&pool, pegawai_id, filter.tahun).await?;
    Ok(Json(kuota))
}