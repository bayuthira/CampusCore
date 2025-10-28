// src/modules/sdm/riwayat_sk_repo.rs
use super::model::{RiwayatSkDb, RiwayatSkPayload, RiwayatSkDetail};
use super::dokumen_model::{DokumenSdmSimple};
use crate::{db::DbPool, errors::AppError};
use futures::future::try_join_all;
use uuid::Uuid;

pub async fn create_repo(
    pool: &DbPool, 
    pegawai_id: Uuid, 
    payload: RiwayatSkPayload
) -> Result<RiwayatSkDetail, AppError> {
    // Insert data riwayat SK
    let item = sqlx::query_as!(
        RiwayatSkDb,
        "INSERT INTO riwayat_sk (pegawai_id, nomor_sk, tanggal_sk, jenis_sk, jabatan, keterangan) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        pegawai_id, 
        payload.nomor_sk, 
        payload.tanggal_sk, 
        payload.jenis_sk, 
        payload.jabatan, 
        payload.keterangan
    )
    .fetch_one(pool)
    .await?;

    // Fetch dokumen terkait (untuk data baru biasanya kosong)
    let dokumen = sqlx::query_as!(
        DokumenSdmSimple,
        r#"
        SELECT id, path_file, kategori as "kategori: _", nama_file_asli 
        FROM dokumen_sdm 
        WHERE entity_id = $1 AND entity_type = 'RiwayatSk'
        "#,
        item.id
    )
    .fetch_all(pool)
    .await?;

    // Return sebagai RiwayatSkDetail
    Ok(RiwayatSkDetail {
        id: item.id,
        pegawai_id: item.pegawai_id,
        nomor_sk: item.nomor_sk,
        tanggal_sk: item.tanggal_sk,
        jenis_sk: item.jenis_sk,
        jabatan: item.jabatan,
        keterangan: item.keterangan,
        dokumen,
    })
}

pub async fn get_all_by_pegawai_id_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
) -> Result<Vec<RiwayatSkDetail>, AppError> {
    // 1. Ambil semua data SK dasar
    let list_db = sqlx::query_as!(
        RiwayatSkDb,
        "SELECT * FROM riwayat_sk WHERE pegawai_id = $1 ORDER BY tanggal_sk DESC",
        pegawai_id
    )
    .fetch_all(pool)
    .await?;

    // 2. Buat future untuk mengambil dokumen - clone pool untuk setiap future
    let futures = list_db.into_iter().map(|item| {
        let pool = pool.clone(); // Clone pool reference
        async move {
            let dokumen = sqlx::query_as!(
                DokumenSdmSimple,
                r#"SELECT id, path_file, kategori as "kategori: _", nama_file_asli 
                   FROM dokumen_sdm 
                   WHERE entity_id = $1 AND entity_type = 'RiwayatSk'"#,
                item.id
            )
            .fetch_all(&pool)
            .await?;

            // Gabungkan
            Ok::<RiwayatSkDetail, AppError>(RiwayatSkDetail {
                id: item.id,
                pegawai_id: item.pegawai_id,
                nomor_sk: item.nomor_sk,
                tanggal_sk: item.tanggal_sk,
                jenis_sk: item.jenis_sk,
                jabatan: item.jabatan,
                keterangan: item.keterangan,
                dokumen,
            })
        }
    });

    // 3. Jalankan semua query
    let list_with_docs = try_join_all(futures).await?;
    Ok(list_with_docs)
}


pub async fn update_repo(
    pool: &DbPool, 
    id: Uuid, 
    payload: RiwayatSkPayload
) -> Result<RiwayatSkDetail, AppError> {
    // Update data riwayat SK
    let item = sqlx::query_as!(
        RiwayatSkDb,
        "UPDATE riwayat_sk SET nomor_sk = $1, tanggal_sk = $2, jenis_sk = $3, jabatan = $4, keterangan = $5 WHERE id = $6 RETURNING *",
        payload.nomor_sk, 
        payload.tanggal_sk, 
        payload.jenis_sk, 
        payload.jabatan, 
        payload.keterangan, 
        id
    )
    .fetch_one(pool)
    .await?;

    // Fetch dokumen terkait
    let dokumen = sqlx::query_as!(
        DokumenSdmSimple,
        r#"
        SELECT id, path_file, kategori as "kategori: _", nama_file_asli 
        FROM dokumen_sdm 
        WHERE entity_id = $1 AND entity_type = 'RiwayatSk'
        "#,
        item.id
    )
    .fetch_all(pool)
    .await?;

    // Return sebagai RiwayatSkDetail
    Ok(RiwayatSkDetail {
        id: item.id,
        pegawai_id: item.pegawai_id,
        nomor_sk: item.nomor_sk,
        tanggal_sk: item.tanggal_sk,
        jenis_sk: item.jenis_sk,
        jabatan: item.jabatan,
        keterangan: item.keterangan,
        dokumen,
    })
}

pub async fn delete_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM riwayat_sk WHERE id = $1", id)
        .execute(pool).await?.rows_affected();
    if rows_affected == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}