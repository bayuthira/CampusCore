use crate::{
    db::DbPool,
    errors::AppError,
};
use super::biaya_model::{BiayaAset, BiayaAsetPayload};
use uuid::Uuid;


pub async fn create_biaya_repo(
    pool: &DbPool,
    user_pencatat_id: Uuid,
    payload: BiayaAsetPayload,
    bukti_url: Option<String>,
) -> Result<BiayaAset, AppError> {
    let tipe_biaya_str = payload.tipe_biaya.as_str();

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO biaya_aset 
        (aset_id, tipe_biaya, deskripsi, jumlah, tanggal_transaksi, vendor, user_pencatat_id, bukti_url) 
        VALUES ($1, $2::"TipeBiaya", $3, $4, $5, $6, $7, $8) RETURNING id
        "#,
    )
    .bind(payload.aset_id).bind(tipe_biaya_str).bind(payload.deskripsi)
    .bind(payload.jumlah).bind(payload.tanggal_transaksi).bind(payload.vendor)
    .bind(user_pencatat_id).bind(bukti_url) // <-- Bind parameter baru
    .fetch_one(pool).await?;

    let new_biaya = get_biaya_by_id_repo(pool, id).await?;
    Ok(new_biaya)
}


pub async fn get_all_biaya_by_aset_id_repo(
    pool: &DbPool,
    aset_id: Uuid,
) -> Result<Vec<BiayaAset>, AppError> {
    let list = sqlx::query_as!(
        BiayaAset,
        r#"
        SELECT 
            b.id, b.aset_id, b.tipe_biaya as "tipe_biaya: _", b.deskripsi, b.jumlah, 
            b.tanggal_transaksi, b.vendor, b.user_pencatat_id,
            COALESCE(u.full_name, 'User Tidak Ditemukan') as "nama_pencatat!",
            b.bukti_url, -- <-- TAMBAHKAN INI
            b.created_at, b.updated_at
        FROM biaya_aset b
        LEFT JOIN users u ON b.user_pencatat_id = u.id
        WHERE b.aset_id = $1
        ORDER BY b.tanggal_transaksi DESC
        "#,
        aset_id
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}


pub async fn get_biaya_by_id_repo(pool: &DbPool, id: Uuid) -> Result<BiayaAset, AppError> {
    let item = sqlx::query_as!(
        BiayaAset,
        r#"
        SELECT 
            b.id, b.aset_id, b.tipe_biaya as "tipe_biaya: _", b.deskripsi, b.jumlah, 
            b.tanggal_transaksi, b.vendor, b.user_pencatat_id,
            COALESCE(u.full_name, 'User Tidak Ditemukan') as "nama_pencatat!",
            b.bukti_url, -- <-- TAMBAHKAN INI
            b.created_at, b.updated_at
        FROM biaya_aset b
        LEFT JOIN users u ON b.user_pencatat_id = u.id
        WHERE b.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}


pub async fn update_biaya_repo(
    pool: &DbPool,
    id: Uuid,
    user_pencatat_id: Uuid,
    payload: BiayaAsetPayload,
) -> Result<BiayaAset, AppError> {
    let tipe_biaya_str = payload.tipe_biaya.as_str();

    sqlx::query(
        r#"
        UPDATE biaya_aset SET
            aset_id = $1, tipe_biaya = $2::"TipeBiaya", deskripsi = $3, jumlah = $4,
            tanggal_transaksi = $5, vendor = $6, user_pencatat_id = $7, updated_at = now()
        WHERE id = $8
        "#,
    )
    .bind(payload.aset_id)
    .bind(tipe_biaya_str)
    .bind(payload.deskripsi)
    .bind(payload.jumlah)
    .bind(payload.tanggal_transaksi)
    .bind(payload.vendor)
    .bind(user_pencatat_id)
    .bind(id)
    .execute(pool)
    .await?;

    let updated_biaya = get_biaya_by_id_repo(pool, id).await?;
    Ok(updated_biaya)
}

pub async fn delete_biaya_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    // 1. Ambil path file SEBELUM menghapus record dari database
    // Kita gunakan `fetch_optional` untuk menangani jika ID tidak ditemukan
    let record_to_delete = sqlx::query!("SELECT bukti_url FROM biaya_aset WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;

    // Jika record tidak ada, kembalikan error `RowNotFound`
    let file_to_delete = match record_to_delete {
        Some(record) => record.bukti_url,
        None => return Err(sqlx::Error::RowNotFound.into()),
    };

    // 2. Hapus record dari database
    sqlx::query!("DELETE FROM biaya_aset WHERE id = $1", id)
        .execute(pool)
        .await?;

    // 3. Jika record tadi memiliki path file, hapus file fisiknya
    if let Some(path_str) = file_to_delete {
        if !path_str.is_empty() {
            // Gunakan `tokio::fs` untuk operasi file asinkron
            if let Err(e) = tokio::fs::remove_file(&path_str).await {
                // Catat error ke log jika file gagal dihapus, tapi jangan gagalkan seluruh request
                // karena data di DB sudah berhasil dihapus.
                tracing::error!("Gagal menghapus file bukti '{}': {}", path_str, e);
            }
        }
    }

    Ok(())
}