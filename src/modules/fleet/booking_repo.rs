use super::booking_model::{
    BookingDetail, BookingFilter, BookingSummary, CreateBookingPayload, EndTripPayload,
    LogPenggunaanDetail, StartTripPayload,
};
use crate::{db::DbPool, errors::AppError};
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn create_booking_repo(
    pool: &DbPool,
    user_pemesan_id: Uuid,
    payload: CreateBookingPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // Cek konflik jadwal kendaraan
    let conflict = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM booking_kendaraan WHERE kendaraan_id = $1 AND status IN ('Disetujui', 'Berlangsung', 'Diajukan') AND (waktu_berangkat, estimasi_waktu_kembali) OVERLAPS ($2, $3))"#,
        payload.kendaraan_id, payload.waktu_berangkat, payload.estimasi_waktu_kembali
    ).fetch_one(&mut *tx).await?;

    if conflict.unwrap_or(false) {
        return Err(AppError::Forbidden(
            "Kendaraan tidak tersedia pada rentang waktu tersebut.".to_string(),
        ));
    }

    // Insert booking baru
    sqlx::query!(
        "INSERT INTO booking_kendaraan (kendaraan_id, user_pemesan_id, tujuan, waktu_berangkat, estimasi_waktu_kembali) VALUES ($1, $2, $3, $4, $5)",
        payload.kendaraan_id, user_pemesan_id, payload.tujuan, payload.waktu_berangkat, payload.estimasi_waktu_kembali
    ).execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_all_bookings_repo(
    pool: &DbPool,
    filter: BookingFilter,
) -> Result<Vec<BookingDetail>, AppError> {
    // Mulai dengan query dasar
    let mut query = sqlx::QueryBuilder::new(
        r#"
    SELECT b.id, b.kendaraan_id, k.nama as nama_kendaraan, b.user_pemesan_id, u.full_name as nama_pemesan,
    b.tujuan, b.waktu_berangkat, b.estimasi_waktu_kembali, b.status
    FROM booking_kendaraan b
    JOIN kendaraan k ON b.kendaraan_id = k.id
    JOIN users u ON b.user_pemesan_id = u.id
"#,
    );

    let mut where_clause_added = false;

    // Tambahkan filter status jika ada
    if let Some(status) = filter.status {
        // --- PERBAIKAN DI SINI ---
        // Gunakan CAST untuk mencocokkan tipe ENUM di database
        query.push(" WHERE b.status = CAST(");
        query.push_bind(status.as_str()); // Kirim sebagai string
        query.push(" AS \"StatusBooking\")");
        // -------------------------
        where_clause_added = true;
    }

    // Tambahkan filter rentang waktu jika ada
    if let (Some(start), Some(end)) = (filter.start, filter.end) {
        if where_clause_added {
            query.push(" AND ");
        } else {
            query.push(" WHERE ");
        }
        query.push("(b.waktu_berangkat, b.estimasi_waktu_kembali) OVERLAPS (");
        query.push_bind(start);
        query.push(", ");
        query.push_bind(end);
        query.push(")");
    }

    query.push(" ORDER BY b.waktu_berangkat DESC");

    let list = query
        .build_query_as::<BookingDetail>()
        .fetch_all(pool)
        .await?;
    Ok(list)
}

pub async fn approve_booking_repo(
    pool: &DbPool,
    id: Uuid,
    user_approve_id: Uuid,
    catatan: Option<String>,
) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE booking_kendaraan SET status = 'Disetujui', user_approve_id = $1, catatan_approval = $2, updated_at = now() WHERE id = $3 AND status = 'Diajukan'",
        user_approve_id, catatan, id
    ).execute(pool).await?;
    Ok(())
}

pub async fn reject_booking_repo(
    pool: &DbPool,
    id: Uuid,
    user_approve_id: Uuid,
    catatan: Option<String>,
) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE booking_kendaraan SET status = 'Ditolak', user_approve_id = $1, catatan_approval = $2, updated_at = now() WHERE id = $3 AND status = 'Diajukan'",
        user_approve_id, catatan, id
    ).execute(pool).await?;
    Ok(())
}

pub async fn start_trip_repo(
    pool: &DbPool,
    booking_id: Uuid,
    payload: StartTripPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    let booking = sqlx::query!(
        "SELECT kendaraan_id FROM booking_kendaraan WHERE id = $1 AND status = 'Disetujui'",
        booking_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::Forbidden("Booking tidak ditemukan atau statusnya bukan 'Disetujui'.".to_string())
    })?;

    // Tentukan timestamp: gunakan dari payload, atau waktu sekarang jika tidak ada
    let berangkat_timestamp = payload
        .waktu_aktual_berangkat
        .unwrap_or_else(OffsetDateTime::now_utc);

    // Buat log penggunaan baru dengan timestamp yang sudah ditentukan
    sqlx::query!(
        "INSERT INTO log_penggunaan (booking_id, odometer_awal, waktu_aktual_berangkat) VALUES ($1, $2, $3)",
        booking_id,
        payload.odometer_awal,
        berangkat_timestamp
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE booking_kendaraan SET status = 'Berlangsung' WHERE id = $1",
        booking_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE kendaraan SET status = 'Digunakan' WHERE id = $1",
        booking.kendaraan_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn end_trip_repo(
    pool: &DbPool,
    booking_id: Uuid,
    payload: EndTripPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    let booking = sqlx::query!(
        "SELECT kendaraan_id FROM booking_kendaraan WHERE id = $1 AND status = 'Berlangsung'",
        booking_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::Forbidden(
            "Booking tidak ditemukan atau statusnya bukan 'Berlangsung'.".to_string(),
        )
    })?;

    // Tentukan timestamp: gunakan dari payload, atau waktu sekarang jika tidak ada
    let kembali_timestamp = payload
        .waktu_aktual_kembali
        .unwrap_or_else(OffsetDateTime::now_utc);

    // Update log penggunaan dengan timestamp yang sudah ditentukan
    sqlx::query!(
        "UPDATE log_penggunaan SET odometer_akhir = $1, bahan_bakar_diisi = $2, catatan_kondisi_kembali = $3, waktu_aktual_kembali = $4 WHERE booking_id = $5",
        payload.odometer_akhir,
        payload.bahan_bakar_diisi,
        payload.catatan_kondisi_kembali,
        kembali_timestamp,
        booking_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE booking_kendaraan SET status = 'Selesai' WHERE id = $1",
        booking_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE kendaraan SET status = 'Tersedia' WHERE id = $1",
        booking.kendaraan_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_my_bookings_repo(
    pool: &DbPool,
    user_pemesan_id: Uuid,
) -> Result<Vec<BookingDetail>, AppError> {
    let list = sqlx::query_as!(
        BookingDetail,
        r#"
        SELECT b.id, b.kendaraan_id, k.nama as nama_kendaraan, b.user_pemesan_id, u.full_name as nama_pemesan,
        b.tujuan, b.waktu_berangkat, b.estimasi_waktu_kembali, b.status as "status: _"
        FROM booking_kendaraan b
        JOIN kendaraan k ON b.kendaraan_id = k.id
        JOIN users u ON b.user_pemesan_id = u.id
        WHERE b.user_pemesan_id = $1
        ORDER BY b.waktu_berangkat DESC
        "#,
        user_pemesan_id
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}

pub async fn get_log_by_booking_id_repo(
    pool: &DbPool,
    booking_id: Uuid,
) -> Result<LogPenggunaanDetail, AppError> {
    let log = sqlx::query_as!(
        LogPenggunaanDetail,
        "SELECT odometer_awal, odometer_akhir, waktu_aktual_berangkat, waktu_aktual_kembali, bahan_bakar_diisi, catatan_kondisi_kembali FROM log_penggunaan WHERE booking_id = $1",
        booking_id
    ).fetch_one(pool).await?;
    Ok(log)
}

pub async fn get_booking_summary_repo(pool: &DbPool) -> Result<BookingSummary, AppError> {
    let summary = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status = 'Diajukan') as "diajukan!",
            COUNT(*) FILTER (WHERE status = 'Disetujui') as "disetujui!",
            COUNT(*) FILTER (WHERE status = 'Ditolak') as "ditolak!",
            COUNT(*) FILTER (WHERE status = 'Dibatalkan') as "dibatalkan!",
            COUNT(*) FILTER (WHERE status = 'Berlangsung') as "berlangsung!",
            COUNT(*) FILTER (WHERE status = 'Selesai') as "selesai!"
        FROM booking_kendaraan
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(BookingSummary {
        diajukan: summary.diajukan,
        disetujui: summary.disetujui,
        ditolak: summary.ditolak,
        dibatalkan: summary.dibatalkan,
        berlangsung: summary.berlangsung,
        selesai: summary.selesai,
    })
}

pub async fn get_bookings_by_kendaraan_id_repo(
    pool: &DbPool,
    kendaraan_id: Uuid,
    filter: BookingFilter,
) -> Result<Vec<BookingDetail>, AppError> {
    // Query dasar dengan filter wajib untuk kendaraan_id
    let mut query = sqlx::QueryBuilder::new(
        r#"
    SELECT b.id, b.kendaraan_id, k.nama as nama_kendaraan, b.user_pemesan_id, u.full_name as nama_pemesan,
    b.tujuan, b.waktu_berangkat, b.estimasi_waktu_kembali, b.status
    FROM booking_kendaraan b
    JOIN kendaraan k ON b.kendaraan_id = k.id
    JOIN users u ON b.user_pemesan_id = u.id
    WHERE b.kendaraan_id = "#,
    );

    query.push_bind(kendaraan_id);

    // --- TAMBAHKAN LOGIKA FILTER STATUS DI SINI ---
    if let Some(status) = filter.status {
        query.push(" AND b.status = CAST(");
        query.push_bind(status.as_str()); // Kirim sebagai string
        query.push(" AS \"StatusBooking\")");
    }
    // ---------------------------------------------

    // Filter rentang waktu (tidak berubah)
    if let (Some(start), Some(end)) = (filter.start, filter.end) {
        query.push(" AND (b.waktu_berangkat, b.estimasi_waktu_kembali) OVERLAPS (");
        query.push_bind(start);
        query.push(", ");
        query.push_bind(end);
        query.push(")");
    }

    query.push(" ORDER BY b.waktu_berangkat DESC");

    let list = query
        .build_query_as::<BookingDetail>()
        .fetch_all(pool)
        .await?;
    Ok(list)
}
