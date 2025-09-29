use crate::{db::DbPool, errors::AppError, modules::aset::ruangan_model::{Ruangan, RuanganPayload,RuanganFilter}};
use uuid::Uuid;

pub async fn create_ruangan_repo(pool: &DbPool, payload: RuanganPayload) -> Result<Ruangan, AppError> {
    let ruangan = sqlx::query_as!(Ruangan, "INSERT INTO ruangan (kode_ruangan, nama_ruangan, kapasitas, panjang, lebar) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        payload.kode_ruangan, payload.nama_ruangan, payload.kapasitas, payload.panjang, payload.lebar
    ).fetch_one(pool).await?;
    Ok(ruangan)
}

pub async fn get_all_ruangan_repo(pool: &DbPool,filter: RuanganFilter) -> Result<Vec<Ruangan>, AppError> {
    let ruangan_list = match filter.q {
        Some(query) => {
            let search_pattern = format!("%{}%", query);
            sqlx::query_as!(
                Ruangan,
                "SELECT * FROM ruangan WHERE kode_ruangan ILIKE $1 OR nama_ruangan ILIKE $1 ORDER BY kode_ruangan ASC",
                search_pattern
            )
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as!(Ruangan, "SELECT * FROM ruangan ORDER BY kode_ruangan ASC")
                .fetch_all(pool)
                .await?
        }
    };
    Ok(ruangan_list)
}

pub async fn get_ruangan_by_id_repo(pool: &DbPool, id: Uuid) -> Result<Ruangan, AppError> {
    let ruangan = sqlx::query_as!(Ruangan, "SELECT * FROM ruangan WHERE id = $1", id).fetch_one(pool).await?;
    Ok(ruangan)
}

pub async fn update_ruangan_repo(pool: &DbPool, id: Uuid, payload: RuanganPayload) -> Result<Ruangan, AppError> {
    let ruangan = sqlx::query_as!(Ruangan, "UPDATE ruangan SET kode_ruangan = $1, nama_ruangan = $2, kapasitas = $3, panjang = $4, lebar = $5, updated_at = now() WHERE id = $6 RETURNING *",
        payload.kode_ruangan, payload.nama_ruangan, payload.kapasitas, payload.panjang, payload.lebar, id
    ).fetch_one(pool).await?;
    Ok(ruangan)
}

pub async fn delete_ruangan_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM ruangan WHERE id = $1", id).execute(pool).await?.rows_affected();
    if rows_affected == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}