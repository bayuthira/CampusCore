use crate::{db::DbPool, errors::AppError, modules::fleet::booking_model::CreateBookingPayload};
use uuid::Uuid;

pub async fn create_booking_repo(pool: &DbPool, user_pemesan_id: Uuid, payload: CreateBookingPayload) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // Cek konflik jadwal kendaraan
    let conflict = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM booking_kendaraan WHERE kendaraan_id = $1 AND status IN ('Disetujui', 'Berlangsung', 'Diajukan') AND (waktu_berangkat, estimasi_waktu_kembali) OVERLAPS ($2, $3))"#,
        payload.kendaraan_id, payload.waktu_berangkat, payload.estimasi_waktu_kembali
    ).fetch_one(&mut *tx).await?;

    if conflict.unwrap_or(false) {
        return Err(AppError::Forbidden("Kendaraan tidak tersedia pada rentang waktu tersebut.".to_string()));
    }

    // Insert booking baru
    sqlx::query!(
        "INSERT INTO booking_kendaraan (kendaraan_id, user_pemesan_id, tujuan, waktu_berangkat, estimasi_waktu_kembali) VALUES ($1, $2, $3, $4, $5)",
        payload.kendaraan_id, user_pemesan_id, payload.tujuan, payload.waktu_berangkat, payload.estimasi_waktu_kembali
    ).execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}