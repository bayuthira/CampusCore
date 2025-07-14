// src/repositories/aset_repo.rs
use crate::{db::DbPool, errors::AppError, models::aset_model::{AsetDetail, AsetPayload}};
use uuid::Uuid;

const SELECT_ASET_DETAIL: &str = r#"
    SELECT 
        a.id, a.nama_aset, a.kode_aset, a.deskripsi, a.tanggal_pembelian,
        a.jenis_aset_id, COALESCE(ja.nama_jenis, 'Jenis Tidak Ditemukan') as "nama_jenis!",
        a.ruangan_id, r.nama_ruangan, r.kode_ruangan,
        a.created_at, a.updated_at
    FROM aset a
    JOIN jenis_aset ja ON a.jenis_aset_id = ja.id
    LEFT JOIN ruangan r ON a.ruangan_id = r.id
"#;

pub async fn create_aset_repo(pool: &DbPool, payload: AsetPayload) -> Result<AsetDetail, AppError> {
    let id = sqlx::query_scalar!(
        "INSERT INTO aset (nama_aset, kode_aset, deskripsi, tanggal_pembelian, jenis_aset_id, ruangan_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        payload.nama_aset, payload.kode_aset, payload.deskripsi, payload.tanggal_pembelian, payload.jenis_aset_id, payload.ruangan_id
    ).fetch_one(pool).await?;
    let new_aset = get_aset_by_id_repo(pool, id).await?;
    Ok(new_aset)
}

pub async fn get_all_aset_repo(pool: &DbPool) -> Result<Vec<AsetDetail>, AppError> {
    let query = format!("{} ORDER BY a.nama_aset ASC", SELECT_ASET_DETAIL);
    let list = sqlx::query_as(&query).fetch_all(pool).await?;
    Ok(list)
}

pub async fn get_aset_by_id_repo(pool: &DbPool, id: Uuid) -> Result<AsetDetail, AppError> {
    let query = format!("{} WHERE a.id = $1", SELECT_ASET_DETAIL);
    let aset = sqlx::query_as(&query).bind(id).fetch_one(pool).await?;
    Ok(aset)
}

pub async fn update_aset_repo(pool: &DbPool, id: Uuid, payload: AsetPayload) -> Result<AsetDetail, AppError> {
    sqlx::query!(
        "UPDATE aset SET nama_aset = $1, kode_aset = $2, deskripsi = $3, tanggal_pembelian = $4, jenis_aset_id = $5, ruangan_id = $6, updated_at = now() WHERE id = $7",
        payload.nama_aset, payload.kode_aset, payload.deskripsi, payload.tanggal_pembelian, payload.jenis_aset_id, payload.ruangan_id, id
    ).execute(pool).await?;
    let updated_aset = get_aset_by_id_repo(pool, id).await?;
    Ok(updated_aset)
}

pub async fn delete_aset_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM aset WHERE id = $1", id).execute(pool).await?.rows_affected();
    if rows_affected == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}