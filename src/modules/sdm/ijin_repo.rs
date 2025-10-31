// src/modules/sdm/ijin_repo.rs
use super::ijin_model::{
    ApprovalIjinPayload, CreatePengajuanIjinPayload, PengajuanIjin, StatusIjin,
};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

/// Helper untuk mengambil satu pengajuan ijin berdasarkan ID
pub async fn get_ijin_by_id_repo<'a, E>(
    executor: E,
    id: Uuid,
) -> Result<PengajuanIjin, AppError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let pengajuan = sqlx::query_as!(
        PengajuanIjin,
        r#"
        SELECT id, pegawai_id, kategori as "kategori: _", tanggal_mulai, tanggal_selesai, 
               alasan, status as "status: _", user_approve_id, catatan_approval, created_at
        FROM pengajuan_ijin
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(executor)
    .await?;
    Ok(pengajuan)
}

/// Endpoint Pegawai: Mengajukan ijin baru
pub async fn create_pengajuan_ijin_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    payload: CreatePengajuanIjinPayload,
) -> Result<PengajuanIjin, AppError> {
    let kategori_str = payload.kategori.as_str();

    // Insert pengajuan ijin baru
    let new_id = sqlx::query_scalar(
        r#"
        INSERT INTO pengajuan_ijin (pegawai_id, kategori, tanggal_mulai, tanggal_selesai, alasan)
        VALUES ($1, $2::"KategoriIjin", $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(pegawai_id)
    .bind(kategori_str)
    .bind(payload.tanggal_mulai)
    .bind(payload.tanggal_selesai)
    .bind(payload.alasan)
    .fetch_one(pool)
    .await?;

    // Ambil data lengkap menggunakan helper
    get_ijin_by_id_repo(pool, new_id).await
}

/// Endpoint Atasan/Admin: Menyetujui pengajuan ijin
pub async fn approve_ijin_repo(
    pool: &DbPool,
    id: Uuid,
    user_approve_id: Uuid,
    payload: ApprovalIjinPayload,
) -> Result<PengajuanIjin, AppError> {
    let status_str = StatusIjin::Disetujui.as_str();

    sqlx::query(
        r#"
        UPDATE pengajuan_ijin
        SET status = $1::"StatusIjin", user_approve_id = $2, catatan_approval = $3, updated_at = now()
        WHERE id = $4 AND status = 'Diajukan'
        "#,
    )
    .bind(status_str)
    .bind(user_approve_id)
    .bind(payload.catatan)
    .bind(id)
    .execute(pool)
    .await?;

    get_ijin_by_id_repo(pool, id).await
}

/// Endpoint Atasan/Admin: Menolak pengajuan ijin
pub async fn reject_ijin_repo(
    pool: &DbPool,
    id: Uuid,
    user_approve_id: Uuid,
    payload: ApprovalIjinPayload,
) -> Result<PengajuanIjin, AppError> {
    let status_str = StatusIjin::Ditolak.as_str();

    sqlx::query(
        r#"
        UPDATE pengajuan_ijin
        SET status = $1::"StatusIjin", user_approve_id = $2, catatan_approval = $3, updated_at = now()
        WHERE id = $4 AND status = 'Diajukan'
        "#,
    )
    .bind(status_str)
    .bind(user_approve_id)
    .bind(payload.catatan)
    .bind(id)
    .execute(pool)
    .await?;

    get_ijin_by_id_repo(pool, id).await
}

/// Endpoint Pegawai: Melihat riwayat ijin milik sendiri
pub async fn get_my_ijin_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
) -> Result<Vec<PengajuanIjin>, AppError> {
    let list = sqlx::query_as!(
        PengajuanIjin,
        r#"
        SELECT id, pegawai_id, kategori as "kategori: _", tanggal_mulai, tanggal_selesai, 
               alasan, status as "status: _", user_approve_id, catatan_approval, created_at
        FROM pengajuan_ijin
        WHERE pegawai_id = $1
        ORDER BY created_at DESC
        "#,
        pegawai_id
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}

/// Endpoint Atasan/Admin: Melihat semua pengajuan ijin (bisa difilter)
pub async fn get_all_ijin_repo(pool: &DbPool) -> Result<Vec<PengajuanIjin>, AppError> {
    // TODO: Tambahkan filter jika perlu
    let list = sqlx::query_as!(
        PengajuanIjin,
        r#"
        SELECT id, pegawai_id, kategori as "kategori: _", tanggal_mulai, tanggal_selesai, 
               alasan, status as "status: _", user_approve_id, catatan_approval, created_at
        FROM pengajuan_ijin
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}