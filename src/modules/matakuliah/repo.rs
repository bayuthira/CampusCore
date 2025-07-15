// src/repositories/matakuliah_repo.rs
use crate::{
    db::DbPool,
    errors::AppError,
    models::matakuliah_model::{CreateMataKuliahPayload, MataKuliahDetail, UpdateMataKuliahPayload},
};
use uuid::Uuid;

pub async fn create_matakuliah_repo(
    pool: &DbPool,
    payload: CreateMataKuliahPayload,
) -> Result<MataKuliahDetail, AppError> {
    let mk_id = sqlx::query_scalar!(
        "INSERT INTO mata_kuliah (kode_mk, nama_mk, sks, semester_target, prodi_id) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        payload.kode_mk, payload.nama_mk, payload.sks, payload.semester_target, payload.prodi_id
    )
    .fetch_one(pool)
    .await?;

    let new_mk = get_matakuliah_by_id_repo(pool, mk_id).await?;
    Ok(new_mk)
}

pub async fn get_all_matakuliah_repo(pool: &DbPool) -> Result<Vec<MataKuliahDetail>, AppError> {
    let mk_list = sqlx::query_as!(
        MataKuliahDetail,
        r#"
        SELECT mk.id, mk.kode_mk, mk.nama_mk, mk.sks, mk.semester_target, mk.prodi_id, p.nama_prodi
        FROM mata_kuliah mk
        LEFT JOIN prodi p ON mk.prodi_id = p.id
        ORDER BY mk.kode_mk ASC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(mk_list)
}

pub async fn get_matakuliah_by_id_repo(pool: &DbPool, id: Uuid) -> Result<MataKuliahDetail, AppError> {
    let mk = sqlx::query_as!(
        MataKuliahDetail,
        r#"
        SELECT mk.id, mk.kode_mk, mk.nama_mk, mk.sks, mk.semester_target, mk.prodi_id, p.nama_prodi
        FROM mata_kuliah mk
        LEFT JOIN prodi p ON mk.prodi_id = p.id
        WHERE mk.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(mk)
}

pub async fn update_matakuliah_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateMataKuliahPayload,
) -> Result<MataKuliahDetail, AppError> {
    // Mulai transaksi
    let mut tx = pool.begin().await?;

    // 1. Update data yang "normal" terlebih dahulu
    sqlx::query!(
        "UPDATE mata_kuliah SET nama_mk = $1, sks = $2, semester_target = $3, prodi_id = $4, updated_at = now() WHERE id = $5",
        payload.nama_mk, payload.sks, payload.semester_target, payload.prodi_id, id
    )
    .execute(&mut *tx)
    .await?;

    // 2. Jika ada `kode_mk` baru di payload, update secara terpisah
    if let Some(new_kode_mk) = payload.kode_mk {
        sqlx::query!(
            "UPDATE mata_kuliah SET kode_mk = $1 WHERE id = $2",
            new_kode_mk,
            id
        )
        .execute(&mut *tx)
        .await?;
    }

    // 3. Commit semua perubahan
    tx.commit().await?;
    
    // Ambil dan kembalikan data terbaru
    let updated_mk = get_matakuliah_by_id_repo(pool, id).await?;
    Ok(updated_mk)
}

pub async fn delete_matakuliah_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM mata_kuliah WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}