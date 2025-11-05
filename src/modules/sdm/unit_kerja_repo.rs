// src/modules/sdm/unit_kerja_repo.rs
use super::unit_kerja_model::{UnitKerja, UnitKerjaPayload};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn create_repo(pool: &DbPool, payload: UnitKerjaPayload) -> Result<UnitKerja, AppError> {
    let item = sqlx::query_as!(
        UnitKerja,
        r#"
        INSERT INTO unit_kerja (induk_unit_id, kode_unit, nama_unit, is_active)
        VALUES ($1, $2, $3, $4)
        RETURNING id, induk_unit_id, kode_unit, nama_unit, is_active
        "#,
        payload.induk_unit_id,
        payload.kode_unit,
        payload.nama_unit,
        payload.is_active.unwrap_or(true)
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}

pub async fn get_all_repo(pool: &DbPool) -> Result<Vec<UnitKerja>, AppError> {
    let list = sqlx::query_as!(
        UnitKerja,
        "SELECT id, induk_unit_id, kode_unit, nama_unit, is_active FROM unit_kerja ORDER BY nama_unit ASC"
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}

pub async fn get_by_id_repo(pool: &DbPool, id: Uuid) -> Result<UnitKerja, AppError> {
    let item = sqlx::query_as!(
        UnitKerja,
        "SELECT id, induk_unit_id, kode_unit, nama_unit, is_active FROM unit_kerja WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}

pub async fn update_repo(pool: &DbPool, id: Uuid, payload: UnitKerjaPayload) -> Result<UnitKerja, AppError> {
    let item = sqlx::query_as!(
        UnitKerja,
        r#"
        UPDATE unit_kerja 
        SET induk_unit_id = $1, kode_unit = $2, nama_unit = $3, is_active = $4
        WHERE id = $5
        RETURNING id, induk_unit_id, kode_unit, nama_unit, is_active
        "#,
        payload.induk_unit_id,
        payload.kode_unit,
        payload.nama_unit,
        payload.is_active.unwrap_or(true),
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(item)
}

pub async fn delete_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows = sqlx::query!("DELETE FROM unit_kerja WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}