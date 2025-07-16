// src/repositories/histori_aset_repo.rs
use super::{
    model::{HistoriAsetDetail,PindahkanAsetPayload,AsetDetail},
};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn get_histori_by_aset_id_repo(pool: &DbPool, aset_id: Uuid) -> Result<Vec<HistoriAsetDetail>, AppError> {
    let histori_list = sqlx::query_as!(
        HistoriAsetDetail,
        r#"
        SELECT 
            h.id,
            h.status as "status: _",
            h.catatan,
            h.tanggal_kejadian,
            h.user_aksi_id,
            u.full_name as "nama_user_aksi!",
            dari.nama_ruangan as dari_ruangan,
            ke.nama_ruangan as ke_ruangan
        FROM histori_aset h
        JOIN users u ON h.user_aksi_id = u.id
        LEFT JOIN ruangan dari ON h.dari_ruangan_id = dari.id
        LEFT JOIN ruangan ke ON h.ke_ruangan_id = ke.id
        WHERE h.aset_id = $1
        ORDER BY h.tanggal_kejadian DESC
        "#,
        aset_id
    )
    .fetch_all(pool)
    .await?;
    Ok(histori_list)
}

pub async fn pindahkan_aset_repo(
    pool: &DbPool,
    aset_id: Uuid,
    user_aksi_id: Uuid,
    payload: PindahkanAsetPayload,
) -> Result<AsetDetail, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Ambil lokasi ruangan saat ini sebelum dipindahkan
    let aset_sebelumnya = sqlx::query!("SELECT ruangan_id FROM aset WHERE id = $1", aset_id)
        .fetch_one(&mut *tx)
        .await?;

    // 2. Buat catatan histori baru
    sqlx::query!(
        r#"
        INSERT INTO histori_aset (aset_id, dari_ruangan_id, ke_ruangan_id, user_aksi_id, status, catatan)
        VALUES ($1, $2, $3, $4, 'Dipindahkan', $5)
        "#,
        aset_id,
        aset_sebelumnya.ruangan_id,
        payload.ke_ruangan_id,
        user_aksi_id,
        payload.catatan
    )
    .execute(&mut *tx)
    .await?;

    // 3. Update lokasi baru di tabel aset
    sqlx::query!(
        "UPDATE aset SET ruangan_id = $1, updated_at = now() WHERE id = $2",
        payload.ke_ruangan_id,
        aset_id
    )
    .execute(&mut *tx)
    .await?;

    // 4. Commit transaksi
    tx.commit().await?;

    // Ambil dan kembalikan detail aset terbaru
    let aset_terbaru = crate::modules::aset::repo::get_aset_by_id_repo(pool, aset_id).await?;
    Ok(aset_terbaru)
}