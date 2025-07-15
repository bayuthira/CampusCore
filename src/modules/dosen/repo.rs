// src/repositories/dosen_repo.rs
use super::model::{CreateDosenPayload, DosenDetail, UpdateDosenPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;



pub async fn create_dosen_repo(
    pool: &DbPool,
    payload: CreateDosenPayload,
) -> Result<DosenDetail, AppError> {
    let mut tx = pool.begin().await?;

    let hashed_password = bcrypt::hash(payload.password, bcrypt::DEFAULT_COST)?;
    
    // Langkah A: Coba buat user dan tangkap hasilnya
    let user_insert_result = sqlx::query!(
        "INSERT INTO users (username, password_hash, full_name, email) VALUES ($1, $2, $3, $4) RETURNING id",
        payload.nidn, // Gunakan NIDN sebagai username
        hashed_password,
        payload.nama_dosen,
        payload.email
    )
    .fetch_one(&mut *tx)
    .await;

    // Langkah B: Periksa hasil pembuatan user, terjemahkan error jika perlu
    let new_user_id = match user_insert_result {
        Ok(record) => record.id, // Jika sukses, ambil ID-nya
        Err(e) => {
            // Jika gagal, periksa apakah ini error duplikasi
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    let constraint = db_err.constraint().unwrap_or_default();
                    if constraint.contains("users_username_key") {
                        // Jika ya, kembalikan AppError spesifik kita dengan pesan yang benar
                        return Err(AppError::DuplicateEntry(format!(
                            "NIDN '{}' sudah terdaftar sebagai user.",
                            payload.nidn
                        )));
                    } else if constraint.contains("users_email_key") {
                        return Err(AppError::DuplicateEntry(format!(
                            "Email '{}' sudah terdaftar.",
                            payload.email.as_deref().unwrap_or_default()
                        )));
                    }
                }
            }
            // Untuk error lain, teruskan saja
            return Err(e.into());
        }
    };

    // Langkah C: Jika user berhasil dibuat, lanjutkan proses
    sqlx::query!(
        "INSERT INTO user_roles (user_id, role_id) VALUES ($1, (SELECT id FROM roles WHERE name = 'DOSEN'))",
        new_user_id
    )
    .execute(&mut *tx)
    .await?;

    let new_dosen_id = sqlx::query_scalar!(
        "INSERT INTO dosen (nidn, nama_dosen, email, prodi_id, user_id) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        payload.nidn,
        payload.nama_dosen,
        payload.email,
        payload.prodi_id,
        new_user_id
    )
    .fetch_one(&mut *tx)
    .await?;
    
    tx.commit().await?;

    let new_dosen = get_dosen_by_id_repo(pool, new_dosen_id).await?;
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