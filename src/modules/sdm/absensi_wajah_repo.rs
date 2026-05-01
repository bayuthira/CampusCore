// src/modules/sdm/absensi_wajah_repo.rs
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn enroll_wajah_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    file_path: String,
) -> Result<(), AppError> {
    let rows_affected = sqlx::query!(
        r#"
        UPDATE pegawai 
        SET foto_wajah_path = $1, 
            status_audit_wajah = 'Menunggu Audit',
            updated_at = now()
        WHERE id = $2
        "#,
        file_path,
        pegawai_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::BadRequest("Pegawai tidak ditemukan.".to_string()));
    }
    Ok(())
}

pub async fn get_status_wajah_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
) -> Result<(Option<String>, String), AppError> {
    let record = sqlx::query!(
        r#"
        SELECT foto_wajah_path, COALESCE(status_audit_wajah, 'Belum Ada') as "status_audit_wajah!" 
        FROM pegawai 
        WHERE id = $1
        "#,
        pegawai_id
    )
    .fetch_optional(pool)
    .await?
    // --- INILAH SUMBER ERROR 500 SEBELUMNYA ---
    // Kita ubah error SQL mentah menjadi pesan Bad Request yang cantik:
    .ok_or_else(|| AppError::BadRequest("Data pegawai tidak ditemukan (Pastikan UUID yang dimasukkan adalah ID Pegawai, bukan ID User!).".to_string()))?;

    Ok((record.foto_wajah_path, record.status_audit_wajah))
}

pub async fn audit_wajah_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    status_audit: String,
) -> Result<(), AppError> {
    let rows_affected = sqlx::query!(
        r#"
        UPDATE pegawai 
        SET status_audit_wajah = $1, updated_at = now()
        WHERE id = $2
        "#,
        status_audit,
        pegawai_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::BadRequest("Pegawai tidak ditemukan.".to_string()));
    }
    Ok(())
}

pub async fn delete_wajah_repo(pool: &DbPool, pegawai_id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!(
        r#"
        UPDATE pegawai 
        SET foto_wajah_path = NULL, status_audit_wajah = 'Belum Ada', updated_at = now()
        WHERE id = $1
        "#,
        pegawai_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::BadRequest("Pegawai tidak ditemukan.".to_string()));
    }
    Ok(())
}
