// src/modules/sdm/penempatan_repo.rs
use super::model::{PenempatanPegawai, PenempatanPegawaiPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

/// Helper untuk mengambil satu data penempatan
async fn get_by_id_inner(
    executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    id: Uuid,
) -> Result<PenempatanPegawai, AppError> {
    let item = sqlx::query_as!(
        PenempatanPegawai,
        r#"
        SELECT 
            pp.id, pp.pegawai_id, pp.unit_kerja_id, 
            uk.nama_unit as "nama_unit_kerja!",
            pp.jabatan, pp.nomor_sk, pp.tanggal_mulai, pp.tanggal_selesai
        FROM penempatan_pegawai pp
        JOIN unit_kerja uk ON pp.unit_kerja_id = uk.id
        WHERE pp.id = $1
        "#,
        id
    )
    .fetch_one(executor)
    .await?;
    Ok(item)
}

/// Mengambil semua riwayat penempatan untuk satu pegawai
pub async fn get_all_by_pegawai_id_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
) -> Result<Vec<PenempatanPegawai>, AppError> {
    let list = sqlx::query_as!(
        PenempatanPegawai,
        r#"
        SELECT 
            pp.id, pp.pegawai_id, pp.unit_kerja_id, 
            uk.nama_unit as "nama_unit_kerja!",
            pp.jabatan, pp.nomor_sk, pp.tanggal_mulai, pp.tanggal_selesai
        FROM penempatan_pegawai pp
        JOIN unit_kerja uk ON pp.unit_kerja_id = uk.id
        WHERE pp.pegawai_id = $1
        ORDER BY pp.tanggal_mulai DESC
        "#,
        pegawai_id
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}

/// Membuat penempatan baru dan mengakhiri penempatan lama (jika ada)
pub async fn create_penempatan_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    payload: PenempatanPegawaiPayload,
) -> Result<PenempatanPegawai, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Akhiri penempatan aktif sebelumnya (jika ada)
    // Penempatan aktif adalah yang `tanggal_selesai`-nya NULL
    sqlx::query!(
        "UPDATE penempatan_pegawai 
         SET tanggal_selesai = $1 
         WHERE pegawai_id = $2 AND tanggal_selesai IS NULL",
        payload.tanggal_mulai, // Tanggal selesai = tanggal mulai penempatan baru
        pegawai_id
    )
    .execute(&mut *tx)
    .await?;

    // 2. Buat penempatan baru
    let new_id = sqlx::query_scalar!(
        r#"
        INSERT INTO penempatan_pegawai (pegawai_id, unit_kerja_id, jabatan, nomor_sk, tanggal_mulai, tanggal_selesai)
        VALUES ($1, $2, $3, $4, $5, NULL)
        RETURNING id
        "#,
        pegawai_id,
        payload.unit_kerja_id,
        payload.jabatan,
        payload.nomor_sk,
        payload.tanggal_mulai
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    // Ambil dan kembalikan data baru yang sudah di-JOIN
    let new_item = get_by_id_inner(pool, new_id).await?;
    Ok(new_item)
}

/// Memperbarui data penempatan (misal: koreksi SK atau tanggal)
pub async fn update_penempatan_repo(
    pool: &DbPool,
    id: Uuid,
    payload: PenempatanPegawaiPayload,
) -> Result<PenempatanPegawai, AppError> {
    // Catatan: Endpoint ini seharusnya tidak mengubah `tanggal_selesai`.
    // `tanggal_selesai` hanya diubah oleh `create_penempatan_repo` baru.
    sqlx::query!(
        "UPDATE penempatan_pegawai 
         SET unit_kerja_id = $1, jabatan = $2, nomor_sk = $3, tanggal_mulai = $4
         WHERE id = $5",
        payload.unit_kerja_id,
        payload.jabatan,
        payload.nomor_sk,
        payload.tanggal_mulai,
        id
    )
    .execute(pool)
    .await?;

    let updated_item = get_by_id_inner(pool, id).await?;
    Ok(updated_item)
}

/// Menghapus data penempatan
pub async fn delete_penempatan_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows = sqlx::query!("DELETE FROM penempatan_pegawai WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}