// src/modules/dosen/repo.rs
use super::model::{CreateDosenPayload, DosenDetail, UpdateDosenPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn create_dosen_repo(
    pool: &DbPool,
    payload: CreateDosenPayload,
) -> Result<DosenDetail, AppError> {
    let mut tx = pool.begin().await?;

    // Pastikan pegawai ada dan ambil user_id-nya (tetap butuh ini untuk insert Role)
    let pegawai = sqlx::query!(
        "SELECT user_id FROM pegawai WHERE id = $1",
        payload.pegawai_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::Forbidden("Pegawai tidak ditemukan".to_string()))?;

    // Insert ke tabel dosen (TANPA nama_dosen, email, dan user_id)
    let new_dosen_id = sqlx::query_scalar!(
        r#"
        INSERT INTO dosen (nidn, prodi_id, pegawai_id, id_penugasan_feeder, ikatan_kerja) 
        VALUES ($1, $2, $3, $4, $5) RETURNING id
        "#,
        payload.nidn,
        payload.prodi_id,
        payload.pegawai_id,
        payload.id_penugasan_feeder,
        payload.ikatan_kerja
    )
    .fetch_one(&mut *tx)
    .await?;

    // Berikan role DOSEN jika dia punya user_id
    if let Some(user_id) = pegawai.user_id {
        sqlx::query!(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'DOSEN')) ON CONFLICT DO NOTHING",
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let new_dosen = get_dosen_by_id_repo(pool, new_dosen_id).await?;
    Ok(new_dosen)
}

// Query untuk mengambil SEMUA dosen dengan JOIN ke tabel pegawai
pub async fn get_all_dosen_repo(pool: &DbPool) -> Result<Vec<DosenDetail>, AppError> {
    let dosen_list = sqlx::query_as!(
        DosenDetail,
        r#"
        SELECT 
            d.id, d.nidn, d.prodi_id as "prodi_id!", d.pegawai_id as "pegawai_id!", 
            d.id_penugasan_feeder, d.ikatan_kerja,
            COALESCE(p.nama_prodi, 'Prodi Tidak Ditemukan') as "nama_prodi!",
            peg.nama_lengkap as "nama_dosen!", peg.email as "email"
        FROM dosen d
        LEFT JOIN prodi p ON d.prodi_id = p.id
        INNER JOIN pegawai peg ON d.pegawai_id = peg.id
        ORDER BY peg.nama_lengkap ASC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(dosen_list)
}

// Helper function untuk mengambil satu dosen berdasarkan ID
pub async fn get_dosen_by_id_repo(pool: &DbPool, id: Uuid) -> Result<DosenDetail, AppError> {
    let dosen = sqlx::query_as!(
        DosenDetail,
        r#"
        SELECT 
            d.id, d.nidn, d.prodi_id as "prodi_id!", d.pegawai_id as "pegawai_id!", 
            d.id_penugasan_feeder, d.ikatan_kerja,
            COALESCE(p.nama_prodi, 'Prodi Tidak Ditemukan') as "nama_prodi!",
            peg.nama_lengkap as "nama_dosen!", peg.email as "email"
        FROM dosen d
        LEFT JOIN prodi p ON d.prodi_id = p.id
        INNER JOIN pegawai peg ON d.pegawai_id = peg.id
        WHERE d.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(dosen)
}

pub async fn update_dosen_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateDosenPayload,
) -> Result<DosenDetail, AppError> {
    let old_dosen = get_dosen_by_id_repo(pool, id).await?;

    // --- PERBAIKAN: Gunakan .or() karena nidn sekarang adalah Option<String> ---
    let upd_nidn = payload.nidn.or(old_dosen.nidn);
    let upd_prodi = payload.prodi_id.unwrap_or(old_dosen.prodi_id);
    let upd_penugasan = payload
        .id_penugasan_feeder
        .or(old_dosen.id_penugasan_feeder);
    let upd_ikatan = payload.ikatan_kerja.or(old_dosen.ikatan_kerja);

    // Update data dosen di database
    sqlx::query!(
        "UPDATE dosen SET nidn = $1, prodi_id = $2, id_penugasan_feeder = $3, ikatan_kerja = $4, updated_at = now() WHERE id = $5",
        upd_nidn,
        upd_prodi,
        upd_penugasan,
        upd_ikatan,
        id
    )
    .execute(pool)
    .await?;

    let updated_dosen = get_dosen_by_id_repo(pool, id).await?;
    Ok(updated_dosen)
}

pub async fn delete_dosen_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM dosen WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}
