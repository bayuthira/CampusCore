use crate::{db::DbPool, errors::AppError, modules::fleet::kendaraan_model::{JenisKendaraan, Kendaraan, KendaraanPayload, StatusKendaraan}};
use uuid::Uuid;

pub async fn create_repo(pool: &DbPool, payload: KendaraanPayload) -> Result<Kendaraan, AppError> {
    let jenis_str = payload.jenis.as_str();

    let id = sqlx::query_scalar(
        r#"
        INSERT INTO kendaraan (jenis, nama, nomor_polisi, merk, model, tahun) 
        VALUES ($1::"JenisKendaraan", $2, $3, $4, $5, $6) RETURNING id
        "#,
    )
    .bind(jenis_str)
    .bind(payload.nama)
    .bind(payload.nomor_polisi)
    .bind(payload.merk)
    .bind(payload.model)
    .bind(payload.tahun)
    .fetch_one(pool)
    .await?;
    
    let new_item = get_by_id_repo(pool, id).await?;
    Ok(new_item)
}

pub async fn get_all_repo(pool: &DbPool) -> Result<Vec<Kendaraan>, AppError> {
    let list = sqlx::query_as!(
        Kendaraan,
        r#"SELECT id, jenis as "jenis: _", nama, nomor_polisi, merk, model, tahun, status as "status: _", created_at, updated_at 
        FROM kendaraan ORDER BY nama ASC"#
    ).fetch_all(pool).await?;
    Ok(list)
}

pub async fn get_by_id_repo(pool: &DbPool, id: Uuid) -> Result<Kendaraan, AppError> {
    let item = sqlx::query_as!(
        Kendaraan,
        r#"SELECT id, jenis as "jenis: _", nama, nomor_polisi, merk, model, tahun, status as "status: _", created_at, updated_at 
        FROM kendaraan WHERE id = $1"#,
        id
    ).fetch_one(pool).await?;
    Ok(item)
}

pub async fn update_repo(pool: &DbPool, id: Uuid, payload: KendaraanPayload) -> Result<Kendaraan, AppError> {
    let jenis_str = payload.jenis.as_str();
    sqlx::query(
        r#"
        UPDATE kendaraan SET jenis = $1::"JenisKendaraan", nama = $2, nomor_polisi = $3, 
        merk = $4, model = $5, tahun = $6, updated_at = now() 
        WHERE id = $7
        "#,
    )
    .bind(jenis_str)
    .bind(payload.nama)
    .bind(payload.nomor_polisi)
    .bind(payload.merk)
    .bind(payload.model)
    .bind(payload.tahun)
    .bind(id)
    .execute(pool)
    .await?;

    let updated_item = get_by_id_repo(pool, id).await?;
    Ok(updated_item)
}

pub async fn delete_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM kendaraan WHERE id = $1", id)
        .execute(pool).await?.rows_affected();
    if rows_affected == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}

// Buat fungsi get_by_id, update, dan delete dengan pola yang sama