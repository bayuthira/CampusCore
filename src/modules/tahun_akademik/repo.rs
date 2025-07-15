// src/repositories/tahun_akademik_repo.rs

use crate::{
    db::DbPool,
    errors::AppError,
    models::tahun_akademik_model::{TaPayload, TahunAkademik},
};
use uuid::Uuid;

// Fungsi helper untuk menonaktifkan semua tahun akademik lain
async fn deactivate_all_other_ta(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id_to_exclude: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    if let Some(id) = id_to_exclude {
        sqlx::query!(
            "UPDATE tahun_akademik SET is_active = false WHERE is_active = true AND id != $1",
            id
        )
        .execute(&mut **tx)
        .await?;
    } else {
        sqlx::query!("UPDATE tahun_akademik SET is_active = false WHERE is_active = true")
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

pub async fn create_tahun_akademik_repo(
    pool: &DbPool,
    payload: TaPayload,
) -> Result<TahunAkademik, AppError> {
    let mut tx = pool.begin().await?;
    if payload.is_active {
        deactivate_all_other_ta(&mut tx, None).await?;
    }
    let ta = sqlx::query_as!(
        TahunAkademik,
        r#"
        INSERT INTO tahun_akademik (nama, tanggal_mulai, tanggal_selesai, krs_mulai, krs_selesai, is_active) 
        VALUES ($1, $2, $3, $4, $5, $6) 
        RETURNING *
        "#,
        payload.nama,
        payload.tanggal_mulai,
        payload.tanggal_selesai,
        payload.krs_mulai,
        payload.krs_selesai,
        payload.is_active
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(ta)
}

pub async fn get_all_tahun_akademik_repo(pool: &DbPool) -> Result<Vec<TahunAkademik>, AppError> {
    let ta_list = sqlx::query_as!(TahunAkademik, "SELECT * FROM tahun_akademik ORDER BY tanggal_mulai DESC")
        .fetch_all(pool)
        .await?;
    Ok(ta_list)
}

pub async fn get_tahun_akademik_by_id_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<TahunAkademik, AppError> {
    let ta = sqlx::query_as!(TahunAkademik, "SELECT * FROM tahun_akademik WHERE id = $1", id)
        .fetch_one(pool)
        .await?;
    Ok(ta)
}

pub async fn update_tahun_akademik_repo(
    pool: &DbPool,
    id: Uuid,
    payload: TaPayload,
) -> Result<TahunAkademik, AppError> {
    let mut tx = pool.begin().await?;
    if payload.is_active {
        deactivate_all_other_ta(&mut tx, Some(id)).await?;
    }
    let ta = sqlx::query_as!(
        TahunAkademik,
        r#"
        UPDATE tahun_akademik 
        SET nama = $1, tanggal_mulai = $2, tanggal_selesai = $3, krs_mulai = $4, krs_selesai = $5, is_active = $6, updated_at = now() 
        WHERE id = $7
        RETURNING *
        "#,
        payload.nama, payload.tanggal_mulai, payload.tanggal_selesai, payload.krs_mulai, payload.krs_selesai, payload.is_active, id
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(ta)
}

pub async fn delete_tahun_akademik_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM tahun_akademik WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}