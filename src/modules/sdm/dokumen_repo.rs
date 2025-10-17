// src/modules/sdm/dokumen_repo.rs
use super::dokumen_model::{DokumenSdmDetail, KategoriDokumen, SdmEntityType};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn create_dokumen_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    entity_id: Uuid,
    entity_type: SdmEntityType,
    kategori: KategoriDokumen,
    nama_file_asli: String,
    path_file: String,
    tipe_mime: String,
    user_uploader_id: Uuid,
) -> Result<DokumenSdmDetail, AppError> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO dokumen_sdm 
        (pegawai_id, entity_id, entity_type, kategori, nama_file_asli, path_file, tipe_mime, user_uploader_id)
        VALUES ($1, $2, $3::"SdmEntityType", $4::"KategoriDokumen", $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(pegawai_id).bind(entity_id).bind(entity_type.as_str())
    .bind(kategori.as_str()).bind(nama_file_asli).bind(path_file)
    .bind(tipe_mime).bind(user_uploader_id)
    .fetch_one(pool).await?;

    // Ambil detail lengkap dari data yang baru dibuat
    let new_dokumen = get_dokumen_by_id_repo(pool, id).await?;
    Ok(new_dokumen)
}

pub async fn get_dokumen_by_id_repo(pool: &DbPool, id: Uuid) -> Result<DokumenSdmDetail, AppError> {
    let doc = sqlx::query_as!(
        DokumenSdmDetail,
        r#"
        SELECT 
            d.id, d.pegawai_id, d.entity_id, d.entity_type as "entity_type: _",
            d.kategori as "kategori: _", d.nama_file_asli, d.path_file, d.tipe_mime,
            d.user_uploader_id, COALESCE(u.full_name, 'User Dihapus') as "nama_uploader!",
            d.created_at
        FROM dokumen_sdm d
        LEFT JOIN users u ON d.user_uploader_id = u.id
        WHERE d.id = $1
        "#,
        id
    ).fetch_one(pool).await?;
    Ok(doc)
}

pub async fn get_all_dokumen_by_entity_repo(
    pool: &DbPool,
    entity_id: Uuid,
    entity_type: SdmEntityType,
) -> Result<Vec<DokumenSdmDetail>, AppError> {
    let docs = sqlx::query_as!(
        DokumenSdmDetail,
        r#"
        SELECT 
            d.id, d.pegawai_id, d.entity_id, d.entity_type as "entity_type: _",
            d.kategori as "kategori: _", d.nama_file_asli, d.path_file, d.tipe_mime,
            d.user_uploader_id, COALESCE(u.full_name, 'User Dihapus') as "nama_uploader!",
            d.created_at
        FROM dokumen_sdm d
        LEFT JOIN users u ON d.user_uploader_id = u.id
        WHERE d.entity_id = $1 AND d.entity_type = $2::"SdmEntityType"
        ORDER BY d.created_at DESC
        "#,
        entity_id,
        entity_type.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(docs)
}

pub async fn delete_dokumen_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    // 1. Ambil path file sebelum dihapus dari DB
    let doc_to_delete = sqlx::query!("SELECT path_file FROM dokumen_sdm WHERE id = $1", id)
        .fetch_optional(pool)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    // 2. Hapus record dari database
    sqlx::query!("DELETE FROM dokumen_sdm WHERE id = $1", id)
        .execute(pool)
        .await?;

    // 3. Hapus file fisik dari server
    if let Err(e) = tokio::fs::remove_file(&doc_to_delete.path_file).await {
        tracing::error!("Gagal menghapus file fisik '{}': {}", doc_to_delete.path_file, e);
    }

    Ok(())
}