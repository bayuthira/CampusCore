use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;
use super::booking_model::{BookingDetail, BookingFilter, StatusBooking,CreateBookingPayload};

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

pub async fn get_all_bookings_repo(pool: &DbPool, filter: BookingFilter) -> Result<Vec<BookingDetail>, AppError> {
    let mut query = sqlx::QueryBuilder::new(r#"
        SELECT b.id, b.kendaraan_id, k.nama as nama_kendaraan, b.user_pemesan_id, u.full_name as nama_pemesan,
        b.tujuan, b.waktu_berangkat, b.estimasi_waktu_kembali, b.status -- <-- PERBAIKI DI SINI
        FROM booking_kendaraan b
        JOIN kendaraan k ON b.kendaraan_id = k.id
        JOIN users u ON b.user_pemesan_id = u.id
    "#);

    if let Some(status) = filter.status {
        query.push(" WHERE b.status = CAST(");
        query.push_bind(status.as_str());
        query.push(" AS \"StatusBooking\")");
    }
    
    query.push(" ORDER BY b.waktu_berangkat DESC");

    let list = query.build_query_as::<BookingDetail>().fetch_all(pool).await?;
    Ok(list)
}


pub async fn approve_booking_repo(pool: &DbPool, id: Uuid, user_approve_id: Uuid, catatan: Option<String>) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE booking_kendaraan SET status = 'Disetujui', user_approve_id = $1, catatan_approval = $2, updated_at = now() WHERE id = $3 AND status = 'Diajukan'",
        user_approve_id, catatan, id
    ).execute(pool).await?;
    Ok(())
}

pub async fn reject_booking_repo(pool: &DbPool, id: Uuid, user_approve_id: Uuid, catatan: Option<String>) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE booking_kendaraan SET status = 'Ditolak', user_approve_id = $1, catatan_approval = $2, updated_at = now() WHERE id = $3 AND status = 'Diajukan'",
        user_approve_id, catatan, id
    ).execute(pool).await?;
    Ok(())
}