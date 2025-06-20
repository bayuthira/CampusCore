// src/repositories/dosen_repo.rs
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::dosen_model::{CreateDosenPayload, DosenDetail, UpdateDosenPayload};
use uuid::Uuid;

pub async fn create_dosen_repo(
    pool: &DbPool,
    payload: CreateDosenPayload,
) -> Result<DosenDetail, AppError> {
    // Karena kita butuh nama prodi untuk response, kita lakukan query setelah insert
    // Di aplikasi nyata yang lebih kompleks, bisa jadi Anda hanya mengembalikan ID
    // atau melakukan ini dalam satu transaksi.

    let dosen_id = sqlx::query_scalar!(
        "INSERT INTO dosen (nidn, nama_dosen, email, prodi_id) VALUES ($1, $2, $3, $4) RETURNING id",
        payload.nidn,
        payload.nama_dosen,
        payload.email,
        payload.prodi_id
    )
    .fetch_one(pool)
    .await?;

    // Ambil detail lengkap dosen yang baru dibuat
    let new_dosen = get_dosen_by_id_repo(pool, dosen_id).await?;
    Ok(new_dosen)
}

// Query untuk mengambil SEMUA dosen dengan detail prodinya menggunakan JOIN
pub async fn get_all_dosen_repo(pool: &DbPool) -> Result<Vec<DosenDetail>, AppError> {
    let dosen_list = sqlx::query_as!(
        DosenDetail,
        r#"
        SELECT 
            d.id, d.nidn, d.nama_dosen, d.email, d.prodi_id, 
            p.nama_prodi
        FROM dosen d
        LEFT JOIN prodi p ON d.prodi_id = p.id
        ORDER BY d.nama_dosen ASC
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
            d.id, d.nidn, d.nama_dosen, d.email, d.prodi_id, 
            p.nama_prodi
        FROM dosen d
        LEFT JOIN prodi p ON d.prodi_id = p.id
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
    // Update data dosen di database
    sqlx::query!(
        "UPDATE dosen SET nama_dosen = $1, email = $2, prodi_id = $3, updated_at = now() WHERE id = $4",
        payload.nama_dosen,
        payload.email,
        payload.prodi_id,
        id
    )
    .execute(pool)
    .await?;

    // Ambil dan kembalikan data dosen yang sudah terupdate
    let updated_dosen = get_dosen_by_id_repo(pool, id).await?;
    Ok(updated_dosen)
}

pub async fn delete_dosen_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM dosen WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    // Jika tidak ada baris yang terhapus, berarti ID tidak ditemukan
    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    Ok(())
}