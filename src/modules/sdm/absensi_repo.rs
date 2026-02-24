// src/modules/sdm/absensi_repo.rs
use super::absensi_model::{
    ClockPayload, LogAbsensi, RekapAbsensiFilter, RekapAbsensiHarian, RekapManualPayload,
    StatusAbsensi, TipeAbsensi,
};
use crate::{db::DbPool, errors::AppError};
use sqlx::Executor;
use time::{Date, Duration, Month, OffsetDateTime};
use uuid::Uuid;

pub async fn get_foto_profil_pegawai(pool: &DbPool, pegawai_id: Uuid) -> Result<String, AppError> {
    let foto = sqlx::query_scalar!(
        r#"
        SELECT nama_file_asli
        FROM dokumen_sdm
        WHERE pegawai_id = $1 AND kategori = 'FotoProfil'
        LIMIT 1
        "#,
        pegawai_id
    )
    .fetch_optional(pool)
    .await?;

    match foto {
        Some(path) => Ok(path),
        None => Err(AppError::NotFound("Foto profil referensi tidak ditemukan. Harap upload foto profil terlebih dahulu sebelum menggunakan fitur absensi wajah.".to_string())),
    }
}

/// Helper untuk mengambil satu log absensi berdasarkan ID
async fn get_log_by_id_repo<'a, E>(executor: E, id: Uuid) -> Result<LogAbsensi, AppError>
where
    E: Executor<'a, Database = sqlx::Postgres>,
{
    let log = sqlx::query_as!(
        LogAbsensi,
        r#"
        SELECT id, pegawai_id, waktu_absensi, tipe_absensi as "tipe_absensi: _", 
               latitude, longitude, alamat_absensi,
               foto_absensi_path, face_confidence_score, is_face_verified
        FROM log_absensi
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(executor)
    .await?;
    Ok(log)
}

/// Endpoint Pegawai: Melakukan Clock In
pub async fn clock_in_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    payload: ClockPayload,
) -> Result<LogAbsensi, AppError> {
    let mut tx = pool.begin().await?;
    let today = OffsetDateTime::now_utc().date();

    let existing_rekap = sqlx::query!(
        "SELECT status::TEXT FROM rekap_absensi_harian WHERE pegawai_id = $1 AND tanggal = $2",
        pegawai_id,
        today 
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(rekap) = existing_rekap {
        let status = rekap.status.unwrap_or_default();
        if status != "Hadir" {
            return Err(AppError::Forbidden(format!(
                "Anda tidak dapat clock-in, status Anda hari ini adalah: {}",
                status
            )));
        }
    } else {
        let status_str = StatusAbsensi::Hadir.as_str();
        sqlx::query(
            "INSERT INTO rekap_absensi_harian (pegawai_id, tanggal, status) VALUES ($1, $2, $3::\"StatusAbsensi\")"
        )
        .bind(pegawai_id)
        .bind(today) 
        .bind(status_str)
        .execute(&mut *tx)
        .await?;
    }

    let tipe_absensi_str = TipeAbsensi::ClockIn.as_str();
    let new_log_id = sqlx::query_scalar(
        r#"
        INSERT INTO log_absensi (
            pegawai_id, waktu_absensi, tipe_absensi, latitude, longitude, alamat_absensi,
            foto_absensi_path, face_confidence_score, is_face_verified
        )
        VALUES ($1, now(), $2::"TipeAbsensi", $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(pegawai_id)
    .bind(tipe_absensi_str)
    .bind(payload.latitude)
    .bind(payload.longitude)
    .bind(payload.alamat_absensi)
    .bind(payload.foto_absensi_path)
    .bind(payload.face_confidence_score)
    .bind(payload.is_face_verified)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    get_log_by_id_repo(pool, new_log_id).await
}


/// Endpoint Pegawai: Melakukan Clock Out
pub async fn clock_out_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    payload: ClockPayload,
) -> Result<LogAbsensi, AppError> {
    let tipe_absensi_str = TipeAbsensi::ClockOut.as_str();
    let new_log_id = sqlx::query_scalar(
        r#"
        INSERT INTO log_absensi (
            pegawai_id, waktu_absensi, tipe_absensi, latitude, longitude, alamat_absensi,
            foto_absensi_path, face_confidence_score, is_face_verified
        )
        VALUES ($1, now(), $2::"TipeAbsensi", $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(pegawai_id)
    .bind(tipe_absensi_str)
    .bind(payload.latitude)
    .bind(payload.longitude)
    .bind(payload.alamat_absensi)
    .bind(payload.foto_absensi_path)
    .bind(payload.face_confidence_score)
    .bind(payload.is_face_verified)
    .fetch_one(pool)
    .await?;

    get_log_by_id_repo(pool, new_log_id).await
}

/// Endpoint Admin: Membuat atau mengoreksi rekap absensi harian
pub async fn create_rekap_manual_repo(
    pool: &DbPool,
    payload: RekapManualPayload,
) -> Result<RekapAbsensiHarian, AppError> {
    let status_str = payload.status.as_str();

    let rekap = sqlx::query_as(
        r#"
        INSERT INTO rekap_absensi_harian (pegawai_id, tanggal, status, keterangan)
        VALUES ($1, $2, $3::"StatusAbsensi", $4)
        ON CONFLICT (pegawai_id, tanggal) DO UPDATE SET
            status = EXCLUDED.status,
            keterangan = EXCLUDED.keterangan
        RETURNING id, pegawai_id, tanggal, status as "status: _", keterangan
        "#,
    )
    .bind(payload.pegawai_id)
    .bind(payload.tanggal)
    .bind(status_str)
    .bind(payload.keterangan)
    .fetch_one(pool)
    .await?;

    Ok(rekap)
}

/// Endpoint Pegawai: Melihat rekap absensi bulanan milik sendiri
pub async fn get_my_rekap_absensi_repo(
    pool: &DbPool,
    pegawai_id: Uuid,
    filter: RekapAbsensiFilter,
) -> Result<Vec<RekapAbsensiHarian>, AppError> {
    // --- PERBAIKAN LOGIKA TANGGAL ---
    let bulan_u8: u8 = filter
        .bulan
        .try_into()
        .map_err(|_| AppError::Forbidden("Bulan tidak valid".to_string()))?;
    let bulan_enum = Month::try_from(bulan_u8)?; // Konversi u8 ke enum Month

    let start_date = Date::from_calendar_date(filter.tahun.into(), bulan_enum, 1)?;
    let end_date = (start_date + Duration::days(32)).replace_day(1)? - Duration::days(1);
    // --- AKHIR PERBAIKAN ---

    let list = sqlx::query_as!(
        RekapAbsensiHarian,
        r#"
        SELECT id, pegawai_id, tanggal, status as "status: _", keterangan
        FROM rekap_absensi_harian
        WHERE pegawai_id = $1 AND tanggal BETWEEN $2 AND $3
        ORDER BY tanggal ASC
        "#,
        pegawai_id,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;
    Ok(list)
}

/// Endpoint Pegawai: Melihat log clock-in/out untuk satu hari
pub async fn get_my_logs_for_day_repo(pool: &DbPool, pegawai_id: Uuid, tanggal: Date) -> Result<Vec<LogAbsensi>, AppError> {
    let start_of_day = tanggal.midnight().assume_utc();
    let end_of_day = (tanggal + Duration::days(1)).midnight().assume_utc();
    let list = sqlx::query_as!(LogAbsensi, r#"SELECT id, pegawai_id, waktu_absensi, tipe_absensi as "tipe_absensi: _", latitude, longitude, alamat_absensi, foto_absensi_path, face_confidence_score, is_face_verified FROM log_absensi WHERE pegawai_id = $1 AND waktu_absensi >= $2 AND waktu_absensi < $3 ORDER BY waktu_absensi ASC"#, pegawai_id, start_of_day, end_of_day).fetch_all(pool).await?;
    Ok(list)
}

/// Endpoint Admin: Melihat rekap absensi bulanan semua pegawai
pub async fn get_all_rekap_absensi_repo(
    pool: &DbPool,
    filter: RekapAbsensiFilter,
) -> Result<Vec<RekapAbsensiHarian>, AppError> {
    // --- PERBAIKAN LOGIKA TANGGAL ---
    let bulan_u8: u8 = filter
        .bulan
        .try_into()
        .map_err(|_| AppError::Forbidden("Bulan tidak valid".to_string()))?;
    let bulan_enum = Month::try_from(bulan_u8)?; // Konversi u8 ke enum Month

    let start_date = Date::from_calendar_date(filter.tahun.into(), bulan_enum, 1)?;
    let end_date = (start_date + Duration::days(32)).replace_day(1)? - Duration::days(1);
    // --- AKHIR PERBAIKAN ---

    let mut query = sqlx::QueryBuilder::new(
        r#"
        SELECT id, pegawai_id, tanggal, status as "status: _", keterangan
        FROM rekap_absensi_harian
        WHERE tanggal BETWEEN "#,
    );
    query.push_bind(start_date);
    query.push(" AND ");
    query.push_bind(end_date);

    if let Some(pegawai_id) = filter.pegawai_id {
        query.push(" AND pegawai_id = ");
        query.push_bind(pegawai_id);
    }

    query.push(" ORDER BY tanggal ASC");

    query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}
