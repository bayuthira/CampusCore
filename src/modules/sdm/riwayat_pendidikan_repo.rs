use super::model::{RiwayatPendidikanDb, RiwayatPendidikanDetail, RiwayatPendidikanPayload};
use super::dokumen_model::{DokumenSdmSimple};
use crate::{db::DbPool, errors::AppError};
use futures::future::try_join_all;
use uuid::Uuid;

pub async fn create_riwayat_pendidikan_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    payload: RiwayatPendidikanPayload,
) -> Result<RiwayatPendidikanDetail, AppError> {
    // Insert data riwayat pendidikan
    let item = sqlx::query_as!(
        RiwayatPendidikanDb,
        r#"
        INSERT INTO riwayat_pendidikan 
            (pegawai_id, jenjang, institusi, jurusan, tahun_lulus)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        pegawai_id,
        payload.jenjang,
        payload.institusi,
        payload.jurusan,
        payload.tahun_lulus
    )
    .fetch_one(pool)
    .await?;

    // Fetch dokumen terkait (untuk data baru biasanya kosong)
    let dokumen = sqlx::query_as!(
        DokumenSdmSimple,
        r#"
        SELECT id, path_file, kategori as "kategori: _", nama_file_asli 
        FROM dokumen_sdm 
        WHERE entity_id = $1 AND entity_type = 'RiwayatPendidikan'
        "#,
        item.id
    )
    .fetch_all(pool)
    .await?;

    // Return sebagai RiwayatPendidikanDetail
    Ok(RiwayatPendidikanDetail {
        id: item.id,
        pegawai_id: item.pegawai_id,
        jenjang: item.jenjang,
        institusi: item.institusi,
        jurusan: item.jurusan,
        tahun_lulus: item.tahun_lulus,
        dokumen,
    })
}


pub async fn get_all_riwayat_pendidikan_by_pegawai_id_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
) -> Result<Vec<RiwayatPendidikanDetail>, AppError> {
    // 1. Ambil semua data riwayat dasar
    let list_db = sqlx::query_as!(
        RiwayatPendidikanDb,
        "SELECT * FROM riwayat_pendidikan WHERE pegawai_id = $1 ORDER BY tahun_lulus DESC",
        pegawai_id
    )
    .fetch_all(pool)
    .await?;

    // 2. Buat future untuk mengambil dokumen untuk setiap riwayat
    let futures = list_db.into_iter().map(|item| {
        let pool = pool.clone();
        async move {
            let dokumen = sqlx::query_as!(
                DokumenSdmSimple,
                r#"SELECT id, path_file, kategori as "kategori: _", nama_file_asli 
                   FROM dokumen_sdm 
                   WHERE entity_id = $1 AND entity_type = 'RiwayatPendidikan'"#,
                item.id
            )
            .fetch_all(&pool)
            .await?;

            // Gabungkan
            Ok::<RiwayatPendidikanDetail, AppError>(RiwayatPendidikanDetail {
                id: item.id,
                pegawai_id: item.pegawai_id,
                jenjang: item.jenjang,
                institusi: item.institusi,
                jurusan: item.jurusan,
                tahun_lulus: item.tahun_lulus,
                dokumen,
            })
        }
    });

    // 3. Jalankan semua query dokumen secara konkuren
    let list_with_docs = try_join_all(futures).await?;
    Ok(list_with_docs)
}

pub async fn update_riwayat_pendidikan_repo(
    pool: &DbPool, 
    id: Uuid, 
    payload: RiwayatPendidikanPayload
) -> Result<RiwayatPendidikanDetail, AppError> {
    // Update data riwayat pendidikan
    let item = sqlx::query_as!(
        RiwayatPendidikanDb,
        "UPDATE riwayat_pendidikan SET jenjang = $1, institusi = $2, jurusan = $3, tahun_lulus = $4 WHERE id = $5 RETURNING *",
        payload.jenjang, 
        payload.institusi, 
        payload.jurusan, 
        payload.tahun_lulus, 
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
        WHERE entity_id = $1 AND entity_type = 'RiwayatPendidikan'
        "#,
        item.id
    )
    .fetch_all(pool)
    .await?;

    // Return sebagai RiwayatPendidikanDetail
    Ok(RiwayatPendidikanDetail {
        id: item.id,
        pegawai_id: item.pegawai_id,
        jenjang: item.jenjang,
        institusi: item.institusi,
        jurusan: item.jurusan,
        tahun_lulus: item.tahun_lulus,
        dokumen,
    })
}

pub async fn delete_riwayat_pendidikan_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM riwayat_pendidikan WHERE id = $1", id)
        .execute(pool).await?.rows_affected();
    if rows_affected == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}