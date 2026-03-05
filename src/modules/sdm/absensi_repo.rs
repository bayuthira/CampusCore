// src/modules/sdm/absensi_repo.rs
use super::{
    absensi_model::{
        ClockPayload, LogAbsensi, RekapAbsensiFilter, RekapAbsensiHarian, RekapManualPayload,
        StatusAbsensi, TipeAbsensi, LaporanAbsensiRow
    },
};
use crate::{db::DbPool, errors::AppError};
use time::{Date, Duration, Month, OffsetDateTime}; 
use uuid::Uuid;
use sqlx::Executor;

/// Fungsi baru: Mengambil nama/path file foto profil dari tabel dokumen_sdm
pub async fn get_foto_profil_pegawai(pool: &DbPool, pegawai_id: Uuid) -> Result<String, AppError> {
    let foto = sqlx::query_scalar!(
        r#"
        SELECT path_file
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
        // <-- Diubah dari NotFound menjadi Forbidden
        None => Err(AppError::Forbidden("Foto profil referensi tidak ditemukan. Harap upload foto profil terlebih dahulu sebelum menggunakan fitur absensi wajah.".to_string())),
    }
}

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

pub async fn create_rekap_manual_repo(pool: &DbPool, payload: RekapManualPayload) -> Result<RekapAbsensiHarian, AppError> {
    let status_str = payload.status.as_str();
    let rekap = sqlx::query_as(r#"INSERT INTO rekap_absensi_harian (pegawai_id, tanggal, status, keterangan) VALUES ($1, $2, $3::"StatusAbsensi", $4) ON CONFLICT (pegawai_id, tanggal) DO UPDATE SET status = EXCLUDED.status, keterangan = EXCLUDED.keterangan RETURNING id, pegawai_id, tanggal, status, keterangan"#).bind(payload.pegawai_id).bind(payload.tanggal).bind(status_str).bind(payload.keterangan).fetch_one(pool).await?;
    Ok(rekap)
}

pub async fn get_my_rekap_absensi_repo(pool: &DbPool, pegawai_id: Uuid, filter: RekapAbsensiFilter) -> Result<Vec<RekapAbsensiHarian>, AppError> {
    let bulan_u8: u8 = filter.bulan.try_into().map_err(|_| AppError::Forbidden("Bulan tidak valid".to_string()))?;
    let bulan_enum = Month::try_from(bulan_u8)?;
    let start_date = Date::from_calendar_date(filter.tahun.into(), bulan_enum, 1)?;
    let end_date = (start_date + Duration::days(32)).replace_day(1)? - Duration::days(1);
    let list = sqlx::query_as!(RekapAbsensiHarian, r#"SELECT id, pegawai_id, tanggal, status as "status: _", keterangan FROM rekap_absensi_harian WHERE pegawai_id = $1 AND tanggal BETWEEN $2 AND $3 ORDER BY tanggal ASC"#, pegawai_id, start_date, end_date).fetch_all(pool).await?;
    Ok(list)
}

pub async fn get_my_logs_for_day_repo(pool: &DbPool, pegawai_id: Uuid, tanggal: Date) -> Result<Vec<LogAbsensi>, AppError> {
    let start_of_day = tanggal.midnight().assume_utc();
    let end_of_day = (tanggal + Duration::days(1)).midnight().assume_utc();
    let list = sqlx::query_as!(LogAbsensi, r#"SELECT id, pegawai_id, waktu_absensi, tipe_absensi as "tipe_absensi: _", latitude, longitude, alamat_absensi, foto_absensi_path, face_confidence_score, is_face_verified FROM log_absensi WHERE pegawai_id = $1 AND waktu_absensi >= $2 AND waktu_absensi < $3 ORDER BY waktu_absensi ASC"#, pegawai_id, start_of_day, end_of_day).fetch_all(pool).await?;
    Ok(list)
}

pub async fn get_all_rekap_absensi_repo(pool: &DbPool, filter: RekapAbsensiFilter) -> Result<Vec<RekapAbsensiHarian>, AppError> {
    let bulan_u8: u8 = filter.bulan.try_into().map_err(|_| AppError::Forbidden("Bulan tidak valid".to_string()))?;
    let bulan_enum = Month::try_from(bulan_u8)?;
    let start_date = Date::from_calendar_date(filter.tahun.into(), bulan_enum, 1)?;
    let end_date = (start_date + Duration::days(32)).replace_day(1)? - Duration::days(1);
    let mut query = sqlx::QueryBuilder::new(r#"SELECT id, pegawai_id, tanggal, status, keterangan FROM rekap_absensi_harian WHERE tanggal BETWEEN "#);
    query.push_bind(start_date);
    query.push(" AND ");
    query.push_bind(end_date);
    if let Some(pegawai_id) = filter.pegawai_id { query.push(" AND pegawai_id = "); query.push_bind(pegawai_id); }
    query.push(" ORDER BY tanggal ASC");
    query.build_query_as().fetch_all(pool).await.map_err(Into::into)
}


pub async fn get_laporan_harian_repo(
    pool: &DbPool, 
    tanggal: Date
) -> Result<Vec<LaporanAbsensiRow>, AppError> {
    let rows = sqlx::query_as!(
        LaporanAbsensiRow,
        r#"
        SELECT 
            p.id as "pegawai_id!",
            p.nama_lengkap as "nama_pegawai!",
            $1::date as "tanggal!",
            MIN(la.waktu_absensi) FILTER (WHERE la.tipe_absensi = 'ClockIn') as clock_in,
            MAX(la.waktu_absensi) FILTER (WHERE la.tipe_absensi = 'ClockOut') as clock_out,
            (SELECT foto_absensi_path FROM log_absensi WHERE pegawai_id = p.id AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = $1 AND tipe_absensi = 'ClockIn' ORDER BY waktu_absensi ASC LIMIT 1) as foto_absensi_path_in,
            (SELECT foto_absensi_path FROM log_absensi WHERE pegawai_id = p.id AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = $1 AND tipe_absensi = 'ClockOut' ORDER BY waktu_absensi DESC LIMIT 1) as foto_absensi_path_out,
            (SELECT latitude FROM log_absensi WHERE pegawai_id = p.id AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = $1 AND tipe_absensi = 'ClockIn' ORDER BY waktu_absensi ASC LIMIT 1) as latitude_in,
            (SELECT longitude FROM log_absensi WHERE pegawai_id = p.id AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = $1 AND tipe_absensi = 'ClockIn' ORDER BY waktu_absensi ASC LIMIT 1) as longitude_in,
            (SELECT latitude FROM log_absensi WHERE pegawai_id = p.id AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = $1 AND tipe_absensi = 'ClockOut' ORDER BY waktu_absensi DESC LIMIT 1) as latitude_out,
            (SELECT longitude FROM log_absensi WHERE pegawai_id = p.id AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = $1 AND tipe_absensi = 'ClockOut' ORDER BY waktu_absensi DESC LIMIT 1) as longitude_out,
            rah.status::TEXT as status_harian,
            (SELECT kategori::TEXT FROM pengajuan_ijin WHERE pegawai_id = p.id AND status = 'Disetujui' AND $1 BETWEEN tanggal_mulai AND tanggal_selesai AND kategori IN ('WFH', 'Dinas Luar') LIMIT 1) as ijin_lokasi
        FROM pegawai p
        LEFT JOIN log_absensi la ON p.id = la.pegawai_id AND DATE(la.waktu_absensi AT TIME ZONE 'Asia/Jakarta') = $1
        LEFT JOIN rekap_absensi_harian rah ON p.id = rah.pegawai_id AND rah.tanggal = $1
        WHERE p.is_active = true
        GROUP BY p.id, p.nama_lengkap, rah.status
        ORDER BY p.nama_lengkap ASC
        "#,
        tanggal
    )
    .fetch_all(pool)
    .await?;
    
    Ok(rows)
}

pub async fn get_laporan_bulanan_repo(
    pool: &DbPool, 
    pegawai_id: Uuid, 
    bulan: i32, 
    tahun: i32
) -> Result<Vec<LaporanAbsensiRow>, AppError> {
    let rows = sqlx::query_as!(
        LaporanAbsensiRow,
        r#"
        WITH days AS (
            SELECT generate_series(
                make_date($2, $1, 1), 
                (make_date($2, $1, 1) + interval '1 month' - interval '1 day')::date,
                '1 day'::interval
            )::date as tanggal
        )
        SELECT 
            $3::uuid as "pegawai_id!",
            COALESCE((SELECT nama_lengkap FROM pegawai WHERE id = $3), 'Unknown') as "nama_pegawai!",
            d.tanggal as "tanggal!",
            MIN(la.waktu_absensi) FILTER (WHERE la.tipe_absensi = 'ClockIn') as clock_in,
            MAX(la.waktu_absensi) FILTER (WHERE la.tipe_absensi = 'ClockOut') as clock_out,
            (SELECT foto_absensi_path FROM log_absensi WHERE pegawai_id = $3 AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = d.tanggal AND tipe_absensi = 'ClockIn' ORDER BY waktu_absensi ASC LIMIT 1) as foto_absensi_path_in,
            (SELECT foto_absensi_path FROM log_absensi WHERE pegawai_id = $3 AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = d.tanggal AND tipe_absensi = 'ClockOut' ORDER BY waktu_absensi DESC LIMIT 1) as foto_absensi_path_out,
            (SELECT latitude FROM log_absensi WHERE pegawai_id = $3 AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = d.tanggal AND tipe_absensi = 'ClockIn' ORDER BY waktu_absensi ASC LIMIT 1) as latitude_in,
            (SELECT longitude FROM log_absensi WHERE pegawai_id = $3 AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = d.tanggal AND tipe_absensi = 'ClockIn' ORDER BY waktu_absensi ASC LIMIT 1) as longitude_in,
            (SELECT latitude FROM log_absensi WHERE pegawai_id = $3 AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = d.tanggal AND tipe_absensi = 'ClockOut' ORDER BY waktu_absensi DESC LIMIT 1) as latitude_out,
            (SELECT longitude FROM log_absensi WHERE pegawai_id = $3 AND DATE(waktu_absensi AT TIME ZONE 'Asia/Jakarta') = d.tanggal AND tipe_absensi = 'ClockOut' ORDER BY waktu_absensi DESC LIMIT 1) as longitude_out,
            rah.status::TEXT as status_harian,
            (SELECT kategori::TEXT FROM pengajuan_ijin WHERE pegawai_id = $3 AND status = 'Disetujui' AND d.tanggal BETWEEN tanggal_mulai AND tanggal_selesai AND kategori IN ('WFH', 'Dinas Luar') LIMIT 1) as ijin_lokasi
        FROM days d
        LEFT JOIN log_absensi la ON la.pegawai_id = $3 AND DATE(la.waktu_absensi AT TIME ZONE 'Asia/Jakarta') = d.tanggal
        LEFT JOIN rekap_absensi_harian rah ON rah.pegawai_id = $3 AND rah.tanggal = d.tanggal
        GROUP BY d.tanggal, rah.status
        ORDER BY d.tanggal ASC
        "#,
        bulan,
        tahun,
        pegawai_id
    )
    .fetch_all(pool)
    .await?;
    
    Ok(rows)
}


/// Fungsi mengecek apakah pegawai memiliki ijin lokasi (WFH/Dinas Luar) pada tanggal tertentu
pub async fn cek_ijin_lokasi_aktif(
    pool: &DbPool,
    pegawai_id: Uuid,
    tanggal: time::Date,
) -> Result<Option<String>, AppError> {
    let kategori = sqlx::query_scalar!(
        r#"
        SELECT kategori::TEXT 
        FROM pengajuan_ijin 
        WHERE pegawai_id = $1 AND status = 'Disetujui' 
          AND $2 BETWEEN tanggal_mulai AND tanggal_selesai 
          AND kategori IN ('WFH', 'Dinas Luar') 
        LIMIT 1
        "#,
        pegawai_id,
        tanggal
    )
    .fetch_optional(pool)
    .await?;

    // GUNAKAN .flatten() UNTUK MELEBUR Option<Option<String>> MENJADI Option<String>
    Ok(kategori.flatten())
}