// src/repositories/jenis_aset_repo.rs

use crate::{
    db::DbPool,
    errors::AppError,
    models::jenis_aset_model::{JenisAset, JenisAsetPayload, KelompokAset},
};
use uuid::Uuid;

// =================================================================
// PENDEKATAN UNTUK OPERASI TULIS (CREATE & UPDATE)
// Menggunakan sqlx::query() (tanpa !) dan .bind() untuk menangani ENUM
// =================================================================

pub async fn create_jenis_aset_repo(
    pool: &DbPool,
    payload: JenisAsetPayload,
) -> Result<JenisAset, AppError> {
    let kelompok_str = payload.kelompok.as_str();

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO jenis_aset (nama_jenis, deskripsi, kelompok) 
        VALUES ($1, $2, $3::"KelompokAset") RETURNING id
        "#,
    )
    .bind(payload.nama_jenis)
    .bind(payload.deskripsi)
    .bind(kelompok_str)
    .fetch_one(pool)
    .await?;

    // Panggil helper get by id yang sudah kita perbaiki
    let new_jenis_aset = get_jenis_aset_by_id_repo(pool, id).await?;
    Ok(new_jenis_aset)
}

pub async fn update_jenis_aset_repo(
    pool: &DbPool,
    id: Uuid,
    payload: JenisAsetPayload,
) -> Result<JenisAset, AppError> {
    let kelompok_str = payload.kelompok.as_str();

    let rows_affected = sqlx::query(
        r#"
        UPDATE jenis_aset SET nama_jenis = $1, deskripsi = $2, kelompok = $3::"KelompokAset", updated_at = now() 
        WHERE id = $4
        "#,
    )
    .bind(payload.nama_jenis)
    .bind(payload.deskripsi)
    .bind(kelompok_str)
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    let updated_jenis_aset = get_jenis_aset_by_id_repo(pool, id).await?;
    Ok(updated_jenis_aset)
}

// =================================================================
// PENDEKATAN UNTUK OPERASI BACA (GET)
// Menggunakan sqlx::query! (dengan !) dan pemetaan manual untuk keamanan
// =================================================================

pub async fn get_all_jenis_aset_repo(pool: &DbPool) -> Result<Vec<JenisAset>, AppError> {
    let records = sqlx::query!(
        r#"
        SELECT 
            id,
            nama_jenis,
            deskripsi,
            kelompok::TEXT as "kelompok", -- Baca ENUM sebagai TEXT
            created_at,
            updated_at
        FROM jenis_aset 
        ORDER BY nama_jenis ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    // Lakukan konversi dari record DB ke struct API secara manual
    let list: Vec<JenisAset> = records
        .into_iter()
        .map(|rec| {
            let kelompok = match rec.kelompok.as_deref() {
                Some("Prasarana") => KelompokAset::Prasarana,
                _ => KelompokAset::Sarana, // Default
            };
            JenisAset {
                id: rec.id,
                nama_jenis: rec.nama_jenis,
                deskripsi: rec.deskripsi,
                kelompok,
                created_at: rec.created_at,
                updated_at: rec.updated_at,
            }
        })
        .collect();
    
    Ok(list)
}

pub async fn get_jenis_aset_by_id_repo(pool: &DbPool, id: Uuid) -> Result<JenisAset, AppError> {
    let rec = sqlx::query!(
        r#"
        SELECT 
            id,
            nama_jenis,
            deskripsi,
            kelompok::TEXT as "kelompok",
            created_at,
            updated_at
        FROM jenis_aset WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    let kelompok = match rec.kelompok.as_deref() {
        Some("Prasarana") => KelompokAset::Prasarana,
        _ => KelompokAset::Sarana,
    };

    let jenis_aset = JenisAset {
        id: rec.id,
        nama_jenis: rec.nama_jenis,
        deskripsi: rec.deskripsi,
        kelompok,
        created_at: rec.created_at,
        updated_at: rec.updated_at,
    };

    Ok(jenis_aset)
}

// Fungsi delete sudah benar dan tidak perlu diubah
pub async fn delete_jenis_aset_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM jenis_aset WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}