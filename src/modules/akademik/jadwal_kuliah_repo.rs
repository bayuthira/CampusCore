// src/modules/akademik/jadwal_kuliah_repo.rs
use super::jadwal_kuliah_model::{
    CreateJadwalKuliahPayload, DayOfWeek, DosenPengampuDetail, ImportJadwalResult,
    JadwalKuliahCsvRecord, JadwalKuliahDetail, JadwalKuliahFilter, PlotJadwalRuanganPayload,
    TimeWithOffset, UpdateJadwalKuliahPayload,
};
use crate::{db::DbPool, errors::AppError};
use rust_decimal::Decimal;
use sqlx::FromRow;
use time::{Duration, Weekday};
use uuid::Uuid;

pub async fn create_jadwal_kuliah_repo(
    pool: &DbPool,
    payload: CreateJadwalKuliahPayload,
) -> Result<Uuid, AppError> {
    let mut tx = pool.begin().await?;
    let hari_str = payload.hari.as_str();

    // Validasi dosen bentrok
    for dosen in &payload.dosen_pengampu {
        let dosen_record = sqlx::query!(
            "SELECT p.nama_lengkap as nama_dosen FROM dosen d JOIN pegawai p ON d.pegawai_id = p.id WHERE d.id = $1", 
            dosen.dosen_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Forbidden(format!("Dosen dengan ID {} tidak ditemukan.", dosen.dosen_id)))?;

        let nama_dosen_bentrok = dosen_record.nama_dosen;

        let conflict = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM jadwal_kuliah jk
                JOIN jadwal_dosen_pengampu jdp ON jk.id = jdp.jadwal_kuliah_id
                WHERE jdp.dosen_id = $1
                  AND jk.tahun_akademik_id = $2
                  AND jk.hari::TEXT = $3
                  AND (jk.jam_mulai, jk.jam_selesai) OVERLAPS ($4, $5)
            )
            "#,
        )
        .bind(dosen.dosen_id)
        .bind(payload.tahun_akademik_id)
        .bind(hari_str)
        .bind(&payload.jam_mulai)
        .bind(&payload.jam_selesai)
        .fetch_one(&mut *tx)
        .await?;

        if conflict {
            return Err(AppError::Forbidden(format!(
                "Jadwal bentrok untuk dosen '{}' pada hari {} jam tersebut.",
                nama_dosen_bentrok, hari_str
            )));
        }
    }

    // Logika Feeder: Jika nama kelas kosong, gunakan kode internal kelas
    let nama_kelas = payload
        .nama_kelas_kuliah
        .unwrap_or_else(|| payload.kelas.clone());

    // Insert jadwal kuliah dengan field baru
    let jadwal_id = sqlx::query_scalar(
        r#"
        INSERT INTO jadwal_kuliah 
        (matakuliah_id, tahun_akademik_id, hari, jam_mulai, jam_selesai, kelas, id_kelas_kuliah_feeder, nama_kelas_kuliah)
        VALUES ($1, $2, $3::"DayOfWeek", $4, $5, $6, $7, $8) RETURNING id
        "#,
    )
    .bind(payload.matakuliah_id)
    .bind(payload.tahun_akademik_id)
    .bind(hari_str)
    .bind(&payload.jam_mulai)
    .bind(&payload.jam_selesai)
    .bind(&payload.kelas)
    .bind(payload.id_kelas_kuliah_feeder)
    .bind(&nama_kelas)
    .fetch_one(&mut *tx)
    .await?;

    for dosen in payload.dosen_pengampu {
        let peran_str = dosen.peran.as_str();

        // Ambil default values untuk SKS dan Tatap Muka
        let sks_sub = dosen
            .sks_substansi_total
            .unwrap_or_else(|| Decimal::from(0));
        let renc_tm = dosen.rencana_tatap_muka.unwrap_or(16);
        let real_tm = dosen.realisasi_tatap_muka.unwrap_or(0);

        sqlx::query(
            r#"
            INSERT INTO jadwal_dosen_pengampu (
                jadwal_kuliah_id, dosen_id, peran, 
                id_aktivitas_mengajar_feeder, sks_substansi_total, rencana_tatap_muka, realisasi_tatap_muka
            ) VALUES ($1, $2, $3::"PeranDosenPengampu", $4, $5, $6, $7)
            "#,
        )
        .bind(jadwal_id)
        .bind(dosen.dosen_id)
        .bind(peran_str)
        .bind(dosen.id_aktivitas_mengajar_feeder)
        .bind(sks_sub)
        .bind(renc_tm)
        .bind(real_tm)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(jadwal_id)
}

pub async fn plot_jadwal_ruangan_repo(
    pool: &DbPool,
    user_pembuat_id: Uuid,
    payload: PlotJadwalRuanganPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM jadwal_ruangan WHERE jadwal_kuliah_id = $1",
        payload.jadwal_kuliah_id
    )
    .execute(&mut *tx)
    .await?;

    #[derive(sqlx::FromRow)]
    struct JadwalInfo {
        hari: String,
        jam_mulai: TimeWithOffset,
        jam_selesai: TimeWithOffset,
        nama_mk: String,
        tanggal_mulai: time::Date,
        tanggal_selesai: time::Date,
    }

    let jadwal = sqlx::query_as::<_, JadwalInfo>(
        r#"
        SELECT jk.hari::TEXT as hari, jk.jam_mulai, jk.jam_selesai,
               mk.nama_mk, ta.tanggal_mulai, ta.tanggal_selesai
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON jk.matakuliah_id = mk.id
        JOIN tahun_akademik ta ON jk.tahun_akademik_id = ta.id
        WHERE jk.id = $1
        "#,
    )
    .bind(payload.jadwal_kuliah_id)
    .fetch_one(&mut *tx)
    .await?;

    let mut instances_to_create = Vec::new();
    let mut current_date = jadwal.tanggal_mulai;
    let target_weekday = match jadwal.hari.as_str() {
        "Senin" => Weekday::Monday,
        "Selasa" => Weekday::Tuesday,
        "Rabu" => Weekday::Wednesday,
        "Kamis" => Weekday::Thursday,
        "Jumat" => Weekday::Friday,
        "Sabtu" => Weekday::Saturday,
        _ => Weekday::Sunday,
    };

    while current_date <= jadwal.tanggal_selesai {
        if current_date.weekday() == target_weekday {
            let waktu_mulai = current_date
                .with_time(jadwal.jam_mulai.time)
                .assume_offset(jadwal.jam_mulai.offset);
            let waktu_selesai = current_date
                .with_time(jadwal.jam_selesai.time)
                .assume_offset(jadwal.jam_selesai.offset);

            instances_to_create.push((waktu_mulai, waktu_selesai));
        }
        current_date += Duration::days(1);
    }

    for (start_time, end_time) in instances_to_create {
        let conflict = sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM jadwal_ruangan WHERE ruangan_id = $1 AND (waktu_mulai, waktu_selesai) OVERLAPS ($2, $3))",
            payload.ruangan_id, start_time, end_time
        ).fetch_one(&mut *tx).await?;

        if conflict.unwrap_or(false) {
            return Err(AppError::Forbidden(format!(
                "Konflik jadwal untuk ruangan pada {}.",
                start_time.date()
            )));
        }

        sqlx::query!(
            "INSERT INTO jadwal_ruangan (ruangan_id, judul_kegiatan, jadwal_kuliah_id, waktu_mulai, waktu_selesai, user_pembuat_id) VALUES ($1, $2, $3, $4, $5, $6)",
            payload.ruangan_id, jadwal.nama_mk, payload.jadwal_kuliah_id, start_time, end_time, user_pembuat_id
        ).execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn unplot_jadwal_ruangan_repo(
    pool: &DbPool,
    jadwal_kuliah_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM jadwal_ruangan WHERE jadwal_kuliah_id = $1",
        jadwal_kuliah_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(FromRow)]
struct JadwalKuliahRow {
    id: Uuid,
    kelas: String,

    // --- TAMBAHAN FEEDER KELAS KULIAH ---
    id_kelas_kuliah_feeder: Option<Uuid>,
    nama_kelas_kuliah: Option<String>,

    hari: DayOfWeek,
    jam_mulai: TimeWithOffset,
    jam_selesai: TimeWithOffset,
    matakuliah_id: Uuid,
    nama_mk: String,
    kode_mk: String,
    sks: i32,
    prodi_id: Uuid,
    nama_prodi: String,
    tahun_akademik_id: Uuid,
    nama_tahun_akademik: String,
    ruangan_id: Option<Uuid>,
    nama_ruangan: Option<String>,
}

pub async fn get_all_jadwal_kuliah_repo(
    pool: &DbPool,
    filter: JadwalKuliahFilter,
) -> Result<Vec<JadwalKuliahDetail>, AppError> {
    let mut query_builder = sqlx::QueryBuilder::new(
        r#"
        SELECT 
            jk.id, jk.kelas, jk.hari, jk.jam_mulai, jk.jam_selesai, 
            jk.id_kelas_kuliah_feeder, jk.nama_kelas_kuliah,
            mk.id as matakuliah_id, mk.nama_mk, mk.kode_mk, mk.sks,
            p.id as prodi_id, p.nama_prodi,
            ta.id as tahun_akademik_id, ta.nama as nama_tahun_akademik,
            jr.ruangan_id, r.nama_ruangan
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON jk.matakuliah_id = mk.id
        JOIN prodi p ON mk.prodi_id = p.id
        JOIN tahun_akademik ta ON jk.tahun_akademik_id = ta.id
        LEFT JOIN jadwal_ruangan jr ON jk.id = jr.jadwal_kuliah_id
        LEFT JOIN ruangan r ON jr.ruangan_id = r.id
        WHERE 1=1
    "#,
    );

    if let Some(prodi_id) = filter.prodi_id {
        query_builder.push(" AND p.id = ");
        query_builder.push_bind(prodi_id);
    }
    if let Some(ta_id) = filter.tahun_akademik_id {
        query_builder.push(" AND ta.id = ");
        query_builder.push_bind(ta_id);
    }

    query_builder.push(" GROUP BY jk.id, mk.id, p.id, ta.id, jr.ruangan_id, r.nama_ruangan");

    let jadwal_rows = query_builder
        .build_query_as::<JadwalKuliahRow>()
        .fetch_all(pool)
        .await?;

    let mut jadwal_details = Vec::new();

    for row in jadwal_rows {
        // --- REVISI QUERY: Ambil list dosen pengampu dengan gabungan gelar dan force NOT NULL (!) ---
        let dosen_pengampu = sqlx::query_as!(
            DosenPengampuDetail,
            r#"
            SELECT 
                d.id as dosen_id, 
                TRIM(COALESCE(NULLIF(TRIM(p.gelar_depan), '') || ' ', '') || TRIM(p.nama_lengkap) || COALESCE(', ' || NULLIF(TRIM(p.gelar_belakang), ''), '')) as "nama_dosen!", 
                jdp.peran as "peran: _",
                jdp.id_aktivitas_mengajar_feeder, 
                jdp.sks_substansi_total as "sks_substansi_total: Decimal", 
                jdp.rencana_tatap_muka as "rencana_tatap_muka: i32", 
                jdp.realisasi_tatap_muka as "realisasi_tatap_muka: i32"
            FROM jadwal_dosen_pengampu jdp
            JOIN dosen d ON jdp.dosen_id = d.id
            JOIN pegawai p ON d.pegawai_id = p.id
            WHERE jdp.jadwal_kuliah_id = $1
            "#,
            row.id
        )
        .fetch_all(pool)
        .await?;

        jadwal_details.push(JadwalKuliahDetail {
            id: row.id,
            kelas: row.kelas,
            id_kelas_kuliah_feeder: row.id_kelas_kuliah_feeder,
            nama_kelas_kuliah: row.nama_kelas_kuliah,
            hari: row.hari,
            jam_mulai: row.jam_mulai,
            jam_selesai: row.jam_selesai,
            matakuliah_id: row.matakuliah_id,
            nama_mk: row.nama_mk,
            kode_mk: row.kode_mk,
            sks: row.sks,
            prodi_id: row.prodi_id,
            nama_prodi: row.nama_prodi,
            tahun_akademik_id: row.tahun_akademik_id,
            nama_tahun_akademik: row.nama_tahun_akademik,
            dosen_pengampu,
            ruangan_id: row.ruangan_id,
            nama_ruangan: row.nama_ruangan,
        });
    }

    Ok(jadwal_details)
}

pub async fn update_jadwal_kuliah_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateJadwalKuliahPayload,
) -> Result<Uuid, AppError> {
    let mut tx = pool.begin().await?;
    let hari_str = payload.hari.as_str();

    let is_duplicate = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM jadwal_kuliah WHERE matakuliah_id = $1 AND tahun_akademik_id = $2 AND kelas = $3 AND id != $4)",
        payload.matakuliah_id, payload.tahun_akademik_id, payload.kelas, id
    ).fetch_one(&mut *tx).await?;

    if is_duplicate.unwrap_or(false) {
        return Err(AppError::DuplicateEntry(
            "Jadwal untuk mata kuliah, tahun akademik, dan kelas ini sudah ada.".to_string(),
        ));
    }

    for dosen in &payload.dosen_pengampu {
        let dosen_record = sqlx::query!(
            "SELECT p.nama_lengkap as nama_dosen FROM dosen d JOIN pegawai p ON d.pegawai_id = p.id WHERE d.id = $1", 
            dosen.dosen_id
        ).fetch_optional(&mut *tx).await?.ok_or_else(|| AppError::Forbidden(format!("Dosen ID {} tidak ditemukan.", dosen.dosen_id)))?;

        let nama_dosen_bentrok = dosen_record.nama_dosen;

        let conflict = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM jadwal_kuliah jk
                JOIN jadwal_dosen_pengampu jdp ON jk.id = jdp.jadwal_kuliah_id
                WHERE jdp.dosen_id = $1
                  AND jk.tahun_akademik_id = $2
                  AND jk.hari::TEXT = $3
                  AND (jk.jam_mulai, jk.jam_selesai) OVERLAPS ($4, $5)
                  AND jk.id != $6
            )
            "#,
        )
        .bind(dosen.dosen_id)
        .bind(payload.tahun_akademik_id)
        .bind(hari_str)
        .bind(&payload.jam_mulai)
        .bind(&payload.jam_selesai)
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        if conflict {
            return Err(AppError::Forbidden(format!(
                "Jadwal bentrok untuk dosen '{}' pada hari {} jam tersebut.",
                nama_dosen_bentrok, hari_str
            )));
        }
    }

    let nama_kelas = payload
        .nama_kelas_kuliah
        .unwrap_or_else(|| payload.kelas.clone());

    sqlx::query(
        r#"
        UPDATE jadwal_kuliah
        SET matakuliah_id=$1, tahun_akademik_id=$2, hari=$3::"DayOfWeek",
            jam_mulai=$4, jam_selesai=$5, kelas=$6, 
            id_kelas_kuliah_feeder=$7, nama_kelas_kuliah=$8, updated_at=now()
        WHERE id=$9
        "#,
    )
    .bind(payload.matakuliah_id)
    .bind(payload.tahun_akademik_id)
    .bind(hari_str)
    .bind(&payload.jam_mulai)
    .bind(&payload.jam_selesai)
    .bind(&payload.kelas)
    .bind(payload.id_kelas_kuliah_feeder)
    .bind(&nama_kelas)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "DELETE FROM jadwal_dosen_pengampu WHERE jadwal_kuliah_id = $1",
        id
    )
    .execute(&mut *tx)
    .await?;

    for dosen in payload.dosen_pengampu {
        let peran_str = dosen.peran.as_str();

        let sks_sub = dosen
            .sks_substansi_total
            .unwrap_or_else(|| Decimal::from(0));
        let renc_tm = dosen.rencana_tatap_muka.unwrap_or(16);
        let real_tm = dosen.realisasi_tatap_muka.unwrap_or(0);

        sqlx::query(
            r#"
            INSERT INTO jadwal_dosen_pengampu (
                jadwal_kuliah_id, dosen_id, peran,
                id_aktivitas_mengajar_feeder, sks_substansi_total, rencana_tatap_muka, realisasi_tatap_muka
            ) VALUES ($1, $2, $3::"PeranDosenPengampu", $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(dosen.dosen_id)
        .bind(peran_str)
        .bind(dosen.id_aktivitas_mengajar_feeder)
        .bind(sks_sub)
        .bind(renc_tm)
        .bind(real_tm)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(id)
}

pub async fn delete_jadwal_kuliah_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM jadwal_kuliah WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}

// =========================================================
// --- FUNGSI IMPORT CSV JADWAL (TERMASUK PLOT RUANGAN) ---
// =========================================================
pub async fn import_jadwal_from_csv_repo(
    pool: &DbPool,
    file_data: bytes::Bytes,
    user_pembuat_id: Uuid,
) -> Result<ImportJadwalResult, AppError> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(file_data.as_ref());

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    // FASE 1: Parsing ke struct sementara
    let mut valid_rows = Vec::new();
    for (index, result) in rdr.deserialize::<JadwalKuliahCsvRecord>().enumerate() {
        match result {
            Ok(row) => valid_rows.push((index + 2, row)),
            Err(e) => {
                failed_count += 1;
                errors.push(format!(
                    "Baris {}: Format CSV tidak valid - {}",
                    index + 2,
                    e
                ));
            }
        }
    }

    let mut tx = pool.begin().await?;

    // FASE 2: Validasi dan Insert ke DB
    for (row_num, row) in valid_rows {
        // Buat savepoint per baris agar jika error tidak merusak insert sebelumnya
        let sp_name = format!("sp_jadwal_{}", row_num);
        sqlx::query(&format!("SAVEPOINT {}", sp_name))
            .execute(&mut *tx)
            .await?;

        // 1. Parsing Hari
        let hari_enum = match row.hari.trim().to_lowercase().as_str() {
            "senin" => DayOfWeek::Senin,
            "selasa" => DayOfWeek::Selasa,
            "rabu" => DayOfWeek::Rabu,
            "kamis" => DayOfWeek::Kamis,
            "jumat" => DayOfWeek::Jumat,
            "sabtu" => DayOfWeek::Sabtu,
            "minggu" => DayOfWeek::Minggu,
            _ => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                    .execute(&mut *tx)
                    .await?;
                failed_count += 1;
                errors.push(format!(
                    "Baris {}: Hari '{}' tidak valid.",
                    row_num, row.hari
                ));
                continue;
            }
        };

        // 2. Parsing Jam (Pisahkan jam mulai dan selesai)
        let jam_parts: Vec<&str> = row.jam.split(|c| c == '-' || c == '–').collect();
        if jam_parts.len() != 2 {
            sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                .execute(&mut *tx)
                .await?;
            failed_count += 1;
            errors.push(format!(
                "Baris {}: Format jam '{}' tidak valid. Gunakan pemisah - atau –",
                row_num, row.jam
            ));
            continue;
        }

        let time_fmt = time::macros::format_description!("[hour]:[minute]");
        let t_mulai = match time::Time::parse(jam_parts[0].trim(), &time_fmt) {
            Ok(t) => t,
            Err(_) => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                    .execute(&mut *tx)
                    .await?;
                failed_count += 1;
                errors.push(format!(
                    "Baris {}: Jam mulai '{}' gagal dibaca.",
                    row_num, jam_parts[0]
                ));
                continue;
            }
        };
        let t_selesai = match time::Time::parse(jam_parts[1].trim(), &time_fmt) {
            Ok(t) => t,
            Err(_) => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                    .execute(&mut *tx)
                    .await?;
                failed_count += 1;
                errors.push(format!(
                    "Baris {}: Jam selesai '{}' gagal dibaca.",
                    row_num, jam_parts[1]
                ));
                continue;
            }
        };

        let offset = time::UtcOffset::from_hms(7, 0, 0).unwrap(); // Default WIB
        let jam_mulai = TimeWithOffset {
            time: t_mulai,
            offset,
        };
        let jam_selesai = TimeWithOffset {
            time: t_selesai,
            offset,
        };

        // 3. Cari Data Mata Kuliah
        // PERBAIKAN E0609: Tambahkan 'kode_mk' ke dalam query SELECT
        let mk = match sqlx::query!(
            "SELECT id, sks, kode_mk FROM mata_kuliah WHERE kode_mk = $1",
            row.kode_mk.trim()
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            Some(m) => m,
            None => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                    .execute(&mut *tx)
                    .await?;
                failed_count += 1;
                errors.push(format!(
                    "Baris {}: Mata Kuliah dengan kode '{}' tidak ditemukan.",
                    row_num, row.kode_mk
                ));
                continue;
            }
        };

        // 4. Cari Data Tahun Akademik (bisa berdasarkan id_feeder 20252 atau nama)
        let ta_str = row.tahun_akademik.trim().to_string();
        let ta = match sqlx::query!("SELECT id, tanggal_mulai, tanggal_selesai FROM tahun_akademik WHERE id_semester_feeder = $1 OR nama = $1 LIMIT 1", ta_str)
            .fetch_optional(&mut *tx).await? {
            Some(t) => t,
            None => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name)).execute(&mut *tx).await?;
                failed_count += 1;
                errors.push(format!("Baris {}: Tahun Akademik '{}' tidak terdaftar.", row_num, row.tahun_akademik));
                continue;
            }
        };

        // 5. Cek Duplikat Jadwal Utama
        let kelas_clean = row.kelas.trim();
        let is_duplicate = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM jadwal_kuliah WHERE matakuliah_id = $1 AND tahun_akademik_id = $2 AND kelas = $3)",
            mk.id, ta.id, kelas_clean
        ).fetch_one(&mut *tx).await?.unwrap_or(false);

        if is_duplicate {
            sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                .execute(&mut *tx)
                .await?;
            failed_count += 1;
            errors.push(format!(
                "Baris {}: Jadwal untuk kelas '{}' MK '{}' sudah ada. Silakan gunakan Update.",
                row_num, kelas_clean, row.kode_mk
            ));
            continue;
        }

        // 6. Buat Jadwal Induk
        let jadwal_id = match sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO jadwal_kuliah 
            (matakuliah_id, tahun_akademik_id, hari, jam_mulai, jam_selesai, kelas, nama_kelas_kuliah)
            VALUES ($1, $2, $3::"DayOfWeek", $4, $5, $6, $7) RETURNING id
            "#
        )
        .bind(mk.id)
        .bind(ta.id)
        .bind(hari_enum.as_str())
        .bind(&jam_mulai)
        .bind(&jam_selesai)
        .bind(kelas_clean)
        .bind(kelas_clean) // Default nama kelas sama dengan kode kelas
        .fetch_one(&mut *tx).await {
            Ok(id) => id,
            Err(e) => {
                sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name)).execute(&mut *tx).await?;
                failed_count += 1;
                errors.push(format!("Baris {}: Gagal menyimpan jadwal: {}", row_num, e));
                continue;
            }
        };

        // 7. Pengolahan Data Dosen Pengampu & SKS
        let dosen_names: Vec<&str> = row.dosen_pengampu.split(" - ").collect();
        if dosen_names.is_empty() || dosen_names[0].trim().is_empty() {
            sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                .execute(&mut *tx)
                .await?;
            failed_count += 1;
            errors.push(format!("Baris {}: Kolom Dosen Pengampu kosong.", row_num));
            continue;
        }

        let n = dosen_names.len() as i32;
        let total_sks_dec = Decimal::from(mk.sks);
        let n_dec = Decimal::from(n);

        // Logika "Bagi rata, sisa berikan ke Koordinator (dosen urutan 1)"
        // PERBAIKAN E0599: Gunakan round_dp(2) sebagai ganti floor_dp
        let base_sks = (total_sks_dec / n_dec).round_dp(2);
        let koordinator_sks = total_sks_dec - (base_sks * Decimal::from(n - 1));

        let base_tm = 16 / n; // Integer division
        let koordinator_tm = base_tm + (16 % n); // Sisa tatap muka untuk koordinator

        let mut dosen_success = true;

        for (i, dosen_str) in dosen_names.iter().enumerate() {
            let is_koordinator = i == 0;
            let peran = if is_koordinator {
                "Koordinator"
            } else {
                "Anggota"
            };
            let sks_substansi = if is_koordinator {
                koordinator_sks
            } else {
                base_sks
            };
            let tatap_muka = if is_koordinator {
                koordinator_tm
            } else {
                base_tm
            };

            let nama_dosen_full = dosen_str.trim();
            // Ambil bagian awal sebelum gelar untuk metode pencarian fuzzy
            let nama_clean = nama_dosen_full
                .split(',')
                .next()
                .unwrap_or(nama_dosen_full)
                .trim();
            let search_pattern = format!("%{}%", nama_clean);

            // Mencari ID Dosen (Prioritas: Exact Match String Gabungan, Fallback: ILIKE nama_lengkap)
            let dosen_id = match sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT d.id 
                FROM dosen d 
                JOIN pegawai p ON d.pegawai_id = p.id 
                WHERE TRIM(COALESCE(NULLIF(TRIM(p.gelar_depan), '') || ' ', '') || TRIM(p.nama_lengkap) || COALESCE(', ' || NULLIF(TRIM(p.gelar_belakang), ''), '')) = $1
                   OR p.nama_lengkap ILIKE $2
                LIMIT 1
                "#
            )
            .bind(nama_dosen_full)
            .bind(&search_pattern)
            .fetch_optional(&mut *tx).await {
                Ok(Some(id)) => id,
                Ok(None) => {
                    errors.push(format!("Baris {}: Dosen dengan nama '{}' tidak ditemukan di database.", row_num, nama_dosen_full));
                    dosen_success = false;
                    break;
                },
                Err(e) => {
                    errors.push(format!("Baris {}: Error saat query dosen '{}': {}", row_num, nama_dosen_full, e));
                    dosen_success = false;
                    break;
                }
            };

            // Masukkan data Dosen ke DB
            if let Err(e) = sqlx::query(
                r#"
                INSERT INTO jadwal_dosen_pengampu (
                    jadwal_kuliah_id, dosen_id, peran, 
                    sks_substansi_total, rencana_tatap_muka, realisasi_tatap_muka
                ) VALUES ($1, $2, $3::"PeranDosenPengampu", $4, $5, 0)
                "#,
            )
            .bind(jadwal_id)
            .bind(dosen_id)
            .bind(peran)
            .bind(sks_substansi)
            .bind(tatap_muka)
            .execute(&mut *tx)
            .await
            {
                errors.push(format!(
                    "Baris {}: Gagal assign dosen '{}': {}",
                    row_num, nama_dosen_full, e
                ));
                dosen_success = false;
                break;
            }
        }

        if !dosen_success {
            sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", sp_name))
                .execute(&mut *tx)
                .await?;
            failed_count += 1;
            continue;
        }

        // 8. Pengolahan Data Ruangan (Plotting Semester Otomatis)
        let nama_ruangan = row.ruangan.trim();
        if !nama_ruangan.is_empty() && nama_ruangan != "-" {
            // Cari Ruangan ID
            let ruangan_id = match sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM ruangan WHERE nama_ruangan ILIKE $1 OR kode_ruangan ILIKE $1 LIMIT 1"
            )
            .bind(nama_ruangan)
            .fetch_optional(&mut *tx).await {
                Ok(Some(id)) => Some(id),
                Ok(None) => {
                    // Jangan digagalkan (Rollback), tapi berikan Warning!
                    errors.push(format!("Baris {}: (Warning) Ruangan '{}' tidak ditemukan. Jadwal dibuat tanpa ruangan ter-plot.", row_num, nama_ruangan));
                    None
                },
                Err(_) => None,
            };

            if let Some(r_id) = ruangan_id {
                let target_weekday = match hari_enum {
                    DayOfWeek::Senin => Weekday::Monday,
                    DayOfWeek::Selasa => Weekday::Tuesday,
                    DayOfWeek::Rabu => Weekday::Wednesday,
                    DayOfWeek::Kamis => Weekday::Thursday,
                    DayOfWeek::Jumat => Weekday::Friday,
                    DayOfWeek::Sabtu => Weekday::Saturday,
                    DayOfWeek::Minggu => Weekday::Sunday,
                };

                let mut current_date = ta.tanggal_mulai;

                // Looping semua hari dalam 1 semester untuk dipesan/diplot
                while current_date <= ta.tanggal_selesai {
                    if current_date.weekday() == target_weekday {
                        let w_mulai = current_date
                            .with_time(jam_mulai.time)
                            .assume_offset(jam_mulai.offset);
                        let w_selesai = current_date
                            .with_time(jam_selesai.time)
                            .assume_offset(jam_selesai.offset);

                        // Insert ignoring conflicts untuk mencegah baris ini error total karena ruangan bertabrakan
                        let _ = sqlx::query!(
                            "INSERT INTO jadwal_ruangan (ruangan_id, judul_kegiatan, jadwal_kuliah_id, waktu_mulai, waktu_selesai, user_pembuat_id) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING",
                            r_id, mk.kode_mk, jadwal_id, w_mulai, w_selesai, user_pembuat_id
                        ).execute(&mut *tx).await;
                    }
                    current_date += Duration::days(1);
                }
            }
        }

        // Sukses! Lepaskan savepoint ini.
        sqlx::query(&format!("RELEASE SAVEPOINT {}", sp_name))
            .execute(&mut *tx)
            .await?;
        success_count += 1;
    }

    if failed_count > 0 && success_count == 0 {
        tx.rollback().await?;
    } else {
        tx.commit().await?; // Simpan yang berhasil
    }

    Ok(ImportJadwalResult {
        status: if failed_count == 0 {
            "SUKSES".to_string()
        } else {
            "SEBAGIAN BERHASIL".to_string()
        },
        total_baris_dipindai: success_count + failed_count,
        baris_berhasil_disimpan: success_count,
        detail_error: errors,
    })
}
