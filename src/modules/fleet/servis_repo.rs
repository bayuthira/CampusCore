use crate::{
    db::DbPool,
    errors::AppError,
    modules::fleet::servis_model::{ServisDetail, ServisPayload,ServisFilter},
};
use uuid::Uuid;

pub async fn create_servis_repo(pool: &DbPool, kendaraan_id: Uuid, user_pencatat_id: Uuid, payload: ServisPayload) -> Result<ServisDetail, AppError> {
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO servis_kendaraan (kendaraan_id, user_pencatat_id, tanggal_servis, odometer_saat_servis, deskripsi, biaya) 
        VALUES ($1, $2, $3, $4, $5, $6) RETURNING id
        "#,
    )
    .bind(kendaraan_id).bind(user_pencatat_id).bind(payload.tanggal_servis)
    .bind(payload.odometer_saat_servis).bind(payload.deskripsi).bind(payload.biaya)
    .fetch_one(pool).await?;
    
    let new_servis = get_servis_by_id_repo(pool, id).await?;
    Ok(new_servis)
}

pub async fn get_all_servis_by_kendaraan_id_repo(
    pool: &DbPool,
    kendaraan_id: Uuid,
    filter: ServisFilter,
) -> Result<Vec<ServisDetail>, AppError> {
    
    let list = if let (Some(start), Some(end)) = (filter.start_date, filter.end_date) {
        // Query JIKA ADA filter tanggal
        sqlx::query_as!(
            ServisDetail,
            r#"
            SELECT s.id, s.kendaraan_id, s.tanggal_servis, s.odometer_saat_servis, s.deskripsi, s.biaya, s.user_pencatat_id,
                   COALESCE(u.full_name, 'User Dihapus') as "nama_pencatat!"
            FROM servis_kendaraan s
            LEFT JOIN users u ON s.user_pencatat_id = u.id
            WHERE s.kendaraan_id = $1 AND s.tanggal_servis BETWEEN $2 AND $3
            ORDER BY s.tanggal_servis DESC
            "#,
            kendaraan_id, start, end
        )
        .fetch_all(pool)
        .await?
    } else {
        // Query JIKA TIDAK ADA filter tanggal
        sqlx::query_as!(
            ServisDetail,
            r#"
            SELECT s.id, s.kendaraan_id, s.tanggal_servis, s.odometer_saat_servis, s.deskripsi, s.biaya, s.user_pencatat_id,
                   COALESCE(u.full_name, 'User Dihapus') as "nama_pencatat!"
            FROM servis_kendaraan s
            LEFT JOIN users u ON s.user_pencatat_id = u.id
            WHERE s.kendaraan_id = $1
            ORDER BY s.tanggal_servis DESC
            "#,
            kendaraan_id
        )
        .fetch_all(pool)
        .await?
    };

    Ok(list)
}


pub async fn get_servis_by_id_repo(pool: &DbPool, id: Uuid) -> Result<ServisDetail, AppError> {
    let item = sqlx::query_as!(
        ServisDetail,
        r#"
        SELECT s.id, s.kendaraan_id, s.tanggal_servis, s.odometer_saat_servis, s.deskripsi, s.biaya, s.user_pencatat_id,
               COALESCE(u.full_name, 'User Dihapus') as "nama_pencatat!"
        FROM servis_kendaraan s
        LEFT JOIN users u ON s.user_pencatat_id = u.id
        WHERE s.id = $1
        "#,
        id
    ).fetch_one(pool).await?;
    Ok(item)
}

pub async fn update_servis_repo(pool: &DbPool, id: Uuid, payload: ServisPayload) -> Result<ServisDetail, AppError> {
    sqlx::query!(
        "UPDATE servis_kendaraan SET tanggal_servis=$1, odometer_saat_servis=$2, deskripsi=$3, biaya=$4, updated_at=now() WHERE id=$5",
        payload.tanggal_servis, payload.odometer_saat_servis, payload.deskripsi, payload.biaya, id
    ).execute(pool).await?;
    
    let updated_item = get_servis_by_id_repo(pool, id).await?;
    Ok(updated_item)
}

pub async fn delete_servis_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM servis_kendaraan WHERE id = $1", id)
        .execute(pool).await?.rows_affected();
    if rows_affected == 0 { return Err(sqlx::Error::RowNotFound.into()); }
    Ok(())
}


