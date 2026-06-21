use super::model::{
    CreatePertemuanPayload, DetailPertemuanResponse, KelasPembelajaran, PertemuanKuliah,
    PresensiMahasiswaRow, SesiPresensiResponse, UpdateBapPayload,
};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

const PERTEMUAN_SELECT: &str = r#"
    SELECT pk.id, pk.jadwal_kuliah_id, pk.rps_mingguan_id, pk.pertemuan_ke,
           pk.tanggal, pk.topik_rencana, pk.topik_realisasi, pk.metode_pembelajaran,
           pk.bap, pk.status::TEXT AS status, pk.dibuka_pada, pk.ditutup_pada
    FROM pertemuan_kuliah pk
"#;

pub async fn get_dosen_id_by_user(pool: &DbPool, user_id: Uuid) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT d.id FROM dosen d JOIN pegawai p ON p.id = d.pegawai_id WHERE p.user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Forbidden("Profil dosen tidak ditemukan.".to_string()))
}

pub async fn assert_dosen_access(
    pool: &DbPool,
    jadwal_id: Uuid,
    dosen_id: Uuid,
) -> Result<(), AppError> {
    let status_rps = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT mk.status_verifikasi_rps
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
        JOIN jadwal_dosen_pengampu jdp ON jdp.jadwal_kuliah_id = jk.id
        WHERE jk.id = $1 AND jdp.dosen_id = $2
        "#,
    )
    .bind(jadwal_id)
    .bind(dosen_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Forbidden("Anda bukan dosen pengampu kelas ini.".to_string()))?;

    if status_rps.as_deref() != Some("Disetujui") {
        return Err(AppError::Forbidden(
            "Proses pembelajaran terkunci sampai RPS disetujui.".to_string(),
        ));
    }
    Ok(())
}

pub async fn get_jadwal_id_by_pertemuan(
    pool: &DbPool,
    pertemuan_id: Uuid,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>("SELECT jadwal_kuliah_id FROM pertemuan_kuliah WHERE id = $1")
        .bind(pertemuan_id)
        .fetch_optional(pool)
        .await?
        .ok_or(sqlx::Error::RowNotFound.into())
}

pub async fn get_kelas_saya(
    pool: &DbPool,
    dosen_id: Uuid,
) -> Result<Vec<KelasPembelajaran>, AppError> {
    let rows = sqlx::query_as::<_, KelasPembelajaran>(
        r#"
        SELECT jk.id AS jadwal_kuliah_id, mk.id AS mata_kuliah_id, mk.kode_mk,
               mk.nama_mk, jk.kelas, ta.nama AS nama_tahun_akademik,
               jk.hari::TEXT AS hari,
               to_char(jk.jam_mulai::TIME, 'HH24:MI') AS jam_mulai,
               to_char(jk.jam_selesai::TIME, 'HH24:MI') AS jam_selesai,
               r.nama_ruangan,
               COALESCE(mk.status_verifikasi_rps, 'Belum Upload') AS status_rps,
               COALESCE(mk.status_verifikasi_rps = 'Disetujui', false) AS pembelajaran_aktif,
               COUNT(pk.id) AS jumlah_pertemuan
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
        JOIN tahun_akademik ta ON ta.id = jk.tahun_akademik_id
        JOIN jadwal_dosen_pengampu jdp ON jdp.jadwal_kuliah_id = jk.id
        LEFT JOIN jadwal_ruangan jr ON jr.jadwal_kuliah_id = jk.id
        LEFT JOIN ruangan r ON r.id = jr.ruangan_id
        LEFT JOIN pertemuan_kuliah pk ON pk.jadwal_kuliah_id = jk.id
        WHERE jdp.dosen_id = $1
        GROUP BY jk.id, mk.id, ta.nama, r.nama_ruangan
        ORDER BY ta.nama DESC, mk.kode_mk, jk.kelas
        "#,
    )
    .bind(dosen_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_pertemuan_by_jadwal(
    pool: &DbPool,
    jadwal_id: Uuid,
) -> Result<Vec<PertemuanKuliah>, AppError> {
    let query = format!(
        "{} WHERE pk.jadwal_kuliah_id = $1 ORDER BY pk.pertemuan_ke",
        PERTEMUAN_SELECT
    );
    Ok(sqlx::query_as::<_, PertemuanKuliah>(&query)
        .bind(jadwal_id)
        .fetch_all(pool)
        .await?)
}

pub async fn create_pertemuan(
    pool: &DbPool,
    jadwal_id: Uuid,
    payload: CreatePertemuanPayload,
) -> Result<PertemuanKuliah, AppError> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO pertemuan_kuliah (
            jadwal_kuliah_id, rps_mingguan_id, pertemuan_ke, tanggal,
            topik_rencana, metode_pembelajaran
        ) VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(jadwal_id)
    .bind(payload.rps_mingguan_id)
    .bind(payload.pertemuan_ke)
    .bind(payload.tanggal)
    .bind(payload.topik_rencana)
    .bind(payload.metode_pembelajaran)
    .fetch_one(pool)
    .await?;
    get_pertemuan_by_id(pool, id).await
}

pub async fn get_pertemuan_by_id(
    pool: &DbPool,
    pertemuan_id: Uuid,
) -> Result<PertemuanKuliah, AppError> {
    let query = format!("{} WHERE pk.id = $1", PERTEMUAN_SELECT);
    Ok(sqlx::query_as::<_, PertemuanKuliah>(&query)
        .bind(pertemuan_id)
        .fetch_one(pool)
        .await?)
}

pub async fn get_detail_pertemuan(
    pool: &DbPool,
    pertemuan_id: Uuid,
) -> Result<DetailPertemuanResponse, AppError> {
    let pertemuan = get_pertemuan_by_id(pool, pertemuan_id).await?;
    let presensi_mahasiswa = sqlx::query_as::<_, PresensiMahasiswaRow>(
        r#"
        SELECT e.id AS enrollment_id, rm.nim, m.nama_mahasiswa,
               COALESCE(pmk.status::TEXT, 'Alpa') AS status,
               pmk.check_in_at, pmk.sumber::TEXT AS sumber, pmk.catatan
        FROM enrollments e
        JOIN registrasi_mahasiswa rm ON rm.id = e.registrasi_id
        JOIN mahasiswa m ON m.id = rm.mahasiswa_id
        JOIN pertemuan_kuliah pk ON pk.jadwal_kuliah_id = e.jadwal_kuliah_id
        LEFT JOIN presensi_mahasiswa_kuliah pmk
            ON pmk.pertemuan_id = pk.id AND pmk.enrollment_id = e.id
        WHERE pk.id = $1 AND e.status_approval::TEXT = 'Disetujui'
        ORDER BY rm.nim
        "#,
    )
    .bind(pertemuan_id)
    .fetch_all(pool)
    .await?;
    Ok(DetailPertemuanResponse {
        pertemuan,
        presensi_mahasiswa,
    })
}

pub async fn update_bap(
    pool: &DbPool,
    pertemuan_id: Uuid,
    payload: UpdateBapPayload,
) -> Result<PertemuanKuliah, AppError> {
    sqlx::query(
        r#"
        UPDATE pertemuan_kuliah SET
            topik_realisasi = $1, metode_pembelajaran = $2, bap = $3, updated_at = now()
        WHERE id = $4 AND status <> 'Ditutup'
        "#,
    )
    .bind(payload.topik_realisasi)
    .bind(payload.metode_pembelajaran)
    .bind(payload.bap)
    .bind(pertemuan_id)
    .execute(pool)
    .await?;
    get_pertemuan_by_id(pool, pertemuan_id).await
}

pub async fn buka_pertemuan(
    pool: &DbPool,
    pertemuan_id: Uuid,
    dosen_id: Uuid,
    user_id: Uuid,
    kode: String,
) -> Result<SesiPresensiResponse, AppError> {
    let mut tx = pool.begin().await?;
    let affected = sqlx::query(
        r#"
        UPDATE pertemuan_kuliah SET status = 'Dibuka', dibuka_oleh = $1,
            dibuka_pada = COALESCE(dibuka_pada, now()), updated_at = now()
        WHERE id = $2 AND status IN ('Dijadwalkan', 'Dibuka')
        "#,
    )
    .bind(user_id)
    .bind(pertemuan_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::BadRequest(
            "Pertemuan yang sudah ditutup atau dibatalkan tidak dapat dibuka kembali.".to_string(),
        ));
    }

    sqlx::query(
        r#"
        INSERT INTO presensi_dosen_kuliah
            (pertemuan_id, dosen_id, status, check_in_at, sumber, dicatat_oleh)
        VALUES ($1, $2, 'Hadir', now(), 'Sistem', $3)
        ON CONFLICT (pertemuan_id, dosen_id) DO UPDATE SET
            status = 'Hadir',
            check_in_at = COALESCE(presensi_dosen_kuliah.check_in_at, now()),
            dicatat_oleh = EXCLUDED.dicatat_oleh,
            updated_at = now()
        "#,
    )
    .bind(pertemuan_id)
    .bind(dosen_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE sesi_presensi_mahasiswa SET aktif = false WHERE pertemuan_id = $1")
        .bind(pertemuan_id)
        .execute(&mut *tx)
        .await?;
    let berlaku_sampai = sqlx::query_scalar::<_, time::OffsetDateTime>(
        r#"
        INSERT INTO sesi_presensi_mahasiswa
            (pertemuan_id, kode, berlaku_sampai, dibuat_oleh)
        VALUES ($1, $2, now() + interval '10 minutes', $3)
        RETURNING berlaku_sampai
        "#,
    )
    .bind(pertemuan_id)
    .bind(&kode)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(SesiPresensiResponse {
        kode,
        berlaku_sampai,
    })
}

pub async fn tutup_pertemuan(
    pool: &DbPool,
    pertemuan_id: Uuid,
    dosen_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let affected = sqlx::query(
        r#"
        UPDATE pertemuan_kuliah SET status = 'Ditutup', ditutup_oleh = $1,
            ditutup_pada = now(), updated_at = now()
        WHERE id = $2 AND status = 'Dibuka'
          AND NULLIF(TRIM(COALESCE(bap, '')), '') IS NOT NULL
          AND NULLIF(TRIM(COALESCE(topik_realisasi, '')), '') IS NOT NULL
        "#,
    )
    .bind(user_id)
    .bind(pertemuan_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::BadRequest(
            "Isi topik realisasi dan BAP sebelum menutup pertemuan.".to_string(),
        ));
    }
    sqlx::query("UPDATE sesi_presensi_mahasiswa SET aktif = false WHERE pertemuan_id = $1")
        .bind(pertemuan_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE presensi_dosen_kuliah SET check_out_at = now(), updated_at = now() WHERE pertemuan_id = $1 AND dosen_id = $2",
    )
    .bind(pertemuan_id)
    .bind(dosen_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        UPDATE jadwal_dosen_pengampu jdp SET realisasi_tatap_muka = (
            SELECT COUNT(*)::INTEGER FROM pertemuan_kuliah pk
            JOIN presensi_dosen_kuliah pdk ON pdk.pertemuan_id = pk.id
            WHERE pk.jadwal_kuliah_id = jdp.jadwal_kuliah_id
              AND pdk.dosen_id = jdp.dosen_id AND pk.status = 'Ditutup'
              AND pdk.status IN ('Hadir', 'Pengganti')
        ) WHERE jdp.dosen_id = $1
          AND jdp.jadwal_kuliah_id = (SELECT jadwal_kuliah_id FROM pertemuan_kuliah WHERE id = $2)
        "#,
    )
    .bind(dosen_id)
    .bind(pertemuan_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn upsert_manual_presensi(
    pool: &DbPool,
    pertemuan_id: Uuid,
    enrollment_id: Uuid,
    status: String,
    catatan: Option<String>,
    user_id: Uuid,
) -> Result<(), AppError> {
    let valid = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM enrollments e
            JOIN pertemuan_kuliah pk ON pk.jadwal_kuliah_id = e.jadwal_kuliah_id
            WHERE e.id = $1 AND pk.id = $2
              AND e.status_approval::TEXT = 'Disetujui'
              AND pk.status = 'Dibuka'
        )
        "#,
    )
    .bind(enrollment_id)
    .bind(pertemuan_id)
    .fetch_one(pool)
    .await?;
    if !valid {
        return Err(AppError::BadRequest(
            "Presensi hanya dapat dicatat saat pertemuan dibuka dan mahasiswa terdaftar pada kelas."
                .to_string(),
        ));
    }
    sqlx::query(
        r#"
        INSERT INTO presensi_mahasiswa_kuliah
            (pertemuan_id, enrollment_id, status, check_in_at, sumber, catatan, dicatat_oleh)
        VALUES (
            $1, $2, $3::"StatusPresensiMahasiswa",
            CASE WHEN $3 IN ('Hadir', 'Terlambat') THEN now() ELSE NULL END,
            'ManualDosen', $4, $5
        )
        ON CONFLICT (pertemuan_id, enrollment_id) DO UPDATE SET
            status = EXCLUDED.status, check_in_at = EXCLUDED.check_in_at,
            sumber = 'ManualDosen', catatan = EXCLUDED.catatan,
            dicatat_oleh = EXCLUDED.dicatat_oleh, updated_at = now()
        "#,
    )
    .bind(pertemuan_id)
    .bind(enrollment_id)
    .bind(status)
    .bind(catatan)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn check_in_mahasiswa(
    pool: &DbPool,
    user_id: Uuid,
    kode: String,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let session = sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"
        SELECT spm.id, spm.pertemuan_id
        FROM sesi_presensi_mahasiswa spm
        JOIN pertemuan_kuliah pk ON pk.id = spm.pertemuan_id
        WHERE UPPER(spm.kode) = UPPER($1) AND spm.aktif = true
          AND spm.berlaku_sampai >= now() AND pk.status = 'Dibuka'
        ORDER BY spm.created_at DESC LIMIT 1
        "#,
    )
    .bind(kode.trim())
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::BadRequest("Kode presensi tidak valid atau sudah kedaluwarsa.".to_string())
    })?;

    let enrollment_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT e.id FROM enrollments e
        JOIN registrasi_mahasiswa rm ON rm.id = e.registrasi_id
        JOIN mahasiswa m ON m.id = rm.mahasiswa_id
        JOIN pertemuan_kuliah pk ON pk.jadwal_kuliah_id = e.jadwal_kuliah_id
        WHERE m.user_id = $1 AND pk.id = $2 AND e.status_approval::TEXT = 'Disetujui'
        "#,
    )
    .bind(user_id)
    .bind(session.1)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::Forbidden("Anda tidak terdaftar pada kelas ini.".to_string()))?;

    let result = sqlx::query(
        r#"
        INSERT INTO presensi_mahasiswa_kuliah
            (pertemuan_id, enrollment_id, status, check_in_at, sumber, dicatat_oleh)
        VALUES ($1, $2, 'Hadir', now(), 'KodeDinamis', $3)
        ON CONFLICT (pertemuan_id, enrollment_id) DO UPDATE SET
            status = 'Hadir',
            check_in_at = now(),
            sumber = 'KodeDinamis',
            catatan = NULL,
            dicatat_oleh = EXCLUDED.dicatat_oleh,
            updated_at = now()
        WHERE presensi_mahasiswa_kuliah.status = 'Alpa'
        "#,
    )
    .bind(session.1)
    .bind(enrollment_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        let current_status = sqlx::query_scalar::<_, String>(
            r#"
            SELECT status::TEXT FROM presensi_mahasiswa_kuliah
            WHERE pertemuan_id = $1 AND enrollment_id = $2
            "#,
        )
        .bind(session.1)
        .bind(enrollment_id)
        .fetch_one(&mut *tx)
        .await?;

        if !matches!(current_status.as_str(), "Hadir" | "Terlambat") {
            return Err(AppError::BadRequest(format!(
                "Presensi sudah ditetapkan dosen dengan status {current_status}. Hubungi dosen untuk melakukan perubahan."
            )));
        }
    }
    tx.commit().await?;
    Ok(())
}
