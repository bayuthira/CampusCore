use super::report_model::{ReportKelasRow, ReportPertemuanRow};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use uuid::Uuid;

fn has_role(claims: &TokenClaims, role: &str) -> bool {
    claims.roles.iter().any(|item| item == role)
}

async fn dosen_id(pool: &DbPool, user_id: Uuid) -> Result<Option<Uuid>, AppError> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT d.id FROM dosen d JOIN pegawai p ON p.id = d.pegawai_id WHERE p.user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?)
}

async fn kaprodi_prodi_ids(pool: &DbPool, user_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    let rows = sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        SELECT COALESCE(NULLIF(ur.context->>'prodi_id', '')::UUID, d.prodi_id)
        FROM user_roles ur
        JOIN roles r ON r.id = ur.role_id AND r.name = 'KAPRODI'
        LEFT JOIN pegawai p ON p.user_id = ur.user_id
        LEFT JOIN dosen d ON d.pegawai_id = p.id
        WHERE ur.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().flatten().collect())
}

async fn scope(
    pool: &DbPool,
    claims: &TokenClaims,
) -> Result<(bool, bool, Vec<Uuid>, Option<Uuid>), AppError> {
    let is_admin = has_role(claims, "SUPER_ADMIN") || has_role(claims, "STAF_AKADEMIK");
    let is_kaprodi = has_role(claims, "KAPRODI");
    let prodi_ids = if is_kaprodi {
        kaprodi_prodi_ids(pool, claims.sub).await?
    } else {
        Vec::new()
    };
    let dosen_id = if has_role(claims, "DOSEN") {
        dosen_id(pool, claims.sub).await?
    } else {
        None
    };
    Ok((is_admin, is_kaprodi, prodi_ids, dosen_id))
}

pub async fn list_report(
    pool: &DbPool,
    claims: &TokenClaims,
    tahun_akademik_id: Uuid,
) -> Result<Vec<ReportKelasRow>, AppError> {
    let (is_admin, is_kaprodi, prodi_ids, dosen_id) = scope(pool, claims).await?;
    Ok(sqlx::query_as::<_, ReportKelasRow>(
        r#"
        WITH meeting_stats AS (
            SELECT pk.jadwal_kuliah_id,
                   COUNT(*) AS jumlah_pertemuan,
                   COUNT(*) FILTER (WHERE pk.status = 'Ditutup') AS pertemuan_ditutup,
                   COUNT(*) FILTER (
                       WHERE pk.status = 'Ditutup'
                         AND NULLIF(TRIM(COALESCE(pk.bap, '')), '') IS NOT NULL
                         AND NULLIF(TRIM(COALESCE(pk.topik_realisasi, '')), '') IS NOT NULL
                   ) AS bap_lengkap
            FROM pertemuan_kuliah pk GROUP BY pk.jadwal_kuliah_id
        ), lecturer_stats AS (
            SELECT jdp.jadwal_kuliah_id,
                   STRING_AGG(DISTINCT p.nama_lengkap, ', ' ORDER BY p.nama_lengkap) AS dosen_pengampu,
                   MAX(COALESCE(jdp.rencana_tatap_muka, 16))::BIGINT AS target_pertemuan
            FROM jadwal_dosen_pengampu jdp
            JOIN dosen d ON d.id = jdp.dosen_id
            JOIN pegawai p ON p.id = d.pegawai_id
            GROUP BY jdp.jadwal_kuliah_id
        ), student_stats AS (
            SELECT e.jadwal_kuliah_id, COUNT(*) AS jumlah_mahasiswa
            FROM enrollments e
            WHERE e.status_approval::TEXT = 'Disetujui'
            GROUP BY e.jadwal_kuliah_id
        ), lecturer_attendance AS (
            SELECT pk.jadwal_kuliah_id,
                   COUNT(DISTINCT pdk.pertemuan_id) FILTER (
                       WHERE pdk.status IN ('Hadir', 'Pengganti')
                   ) AS presensi_dosen
            FROM pertemuan_kuliah pk
            LEFT JOIN presensi_dosen_kuliah pdk ON pdk.pertemuan_id = pk.id
            GROUP BY pk.jadwal_kuliah_id
        ), student_attendance AS (
            SELECT pk.jadwal_kuliah_id,
                   COUNT(pmk.id) FILTER (
                       WHERE pk.status = 'Ditutup' AND pmk.status IN ('Hadir', 'Terlambat')
                   ) AS mahasiswa_hadir
            FROM pertemuan_kuliah pk
            LEFT JOIN presensi_mahasiswa_kuliah pmk ON pmk.pertemuan_id = pk.id
            GROUP BY pk.jadwal_kuliah_id
        )
        SELECT jk.id AS jadwal_kuliah_id, mk.kode_mk, mk.nama_mk, jk.kelas,
               p.nama_prodi, ta.nama AS tahun_akademik,
               COALESCE(ls.dosen_pengampu, '-') AS dosen_pengampu,
               COALESCE(mk.status_verifikasi_rps, 'Belum Upload') AS status_rps,
               COALESCE(ls.target_pertemuan, 16) AS target_pertemuan,
               COALESCE(ms.jumlah_pertemuan, 0) AS jumlah_pertemuan,
               COALESCE(ms.pertemuan_ditutup, 0) AS pertemuan_ditutup,
               COALESCE(ms.bap_lengkap, 0) AS bap_lengkap,
               COALESCE(la.presensi_dosen, 0) AS presensi_dosen,
               COALESCE(ss.jumlah_mahasiswa, 0) AS jumlah_mahasiswa,
               COALESCE(sa.mahasiswa_hadir, 0) AS mahasiswa_hadir,
               (COALESCE(ms.pertemuan_ditutup, 0) * COALESCE(ss.jumlah_mahasiswa, 0))::BIGINT
                   AS total_slot_presensi
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
        JOIN prodi p ON p.id = mk.prodi_id
        JOIN tahun_akademik ta ON ta.id = jk.tahun_akademik_id
        LEFT JOIN meeting_stats ms ON ms.jadwal_kuliah_id = jk.id
        LEFT JOIN lecturer_stats ls ON ls.jadwal_kuliah_id = jk.id
        LEFT JOIN student_stats ss ON ss.jadwal_kuliah_id = jk.id
        LEFT JOIN lecturer_attendance la ON la.jadwal_kuliah_id = jk.id
        LEFT JOIN student_attendance sa ON sa.jadwal_kuliah_id = jk.id
        WHERE jk.tahun_akademik_id = $1
          AND (
              $2 OR ($3 AND mk.prodi_id = ANY($4)) OR EXISTS(
                  SELECT 1 FROM jadwal_dosen_pengampu own
                  WHERE own.jadwal_kuliah_id = jk.id AND own.dosen_id = $5
              )
          )
        ORDER BY p.nama_prodi, mk.kode_mk, jk.kelas
        "#,
    )
    .bind(tahun_akademik_id)
    .bind(is_admin)
    .bind(is_kaprodi)
    .bind(&prodi_ids)
    .bind(dosen_id)
    .fetch_all(pool)
    .await?)
}

async fn assert_report_access(
    pool: &DbPool,
    claims: &TokenClaims,
    jadwal_id: Uuid,
) -> Result<(), AppError> {
    let (is_admin, is_kaprodi, prodi_ids, dosen_id) = scope(pool, claims).await?;
    let allowed = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM jadwal_kuliah jk
            JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
            WHERE jk.id = $1 AND (
                $2 OR ($3 AND mk.prodi_id = ANY($4)) OR EXISTS(
                    SELECT 1 FROM jadwal_dosen_pengampu jdp
                    WHERE jdp.jadwal_kuliah_id = jk.id AND jdp.dosen_id = $5
                )
            )
        )
        "#,
    )
    .bind(jadwal_id)
    .bind(is_admin)
    .bind(is_kaprodi)
    .bind(&prodi_ids)
    .bind(dosen_id)
    .fetch_one(pool)
    .await?;
    if allowed {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Anda tidak memiliki akses ke laporan kelas ini.".to_string(),
        ))
    }
}

pub async fn detail_report(
    pool: &DbPool,
    claims: &TokenClaims,
    jadwal_id: Uuid,
) -> Result<Vec<ReportPertemuanRow>, AppError> {
    assert_report_access(pool, claims, jadwal_id).await?;
    Ok(sqlx::query_as::<_, ReportPertemuanRow>(
        r#"
        SELECT pk.id, pk.pertemuan_ke, pk.tanggal, pk.status::TEXT AS status,
               pk.topik_rencana, pk.topik_realisasi,
               (NULLIF(TRIM(COALESCE(pk.bap, '')), '') IS NOT NULL
                AND NULLIF(TRIM(COALESCE(pk.topik_realisasi, '')), '') IS NOT NULL) AS bap_lengkap,
               COUNT(e.id) FILTER (WHERE pmk.status = 'Hadir') AS hadir,
               COUNT(e.id) FILTER (WHERE pmk.status = 'Terlambat') AS terlambat,
               COUNT(e.id) FILTER (WHERE pmk.status = 'Izin') AS izin,
               COUNT(e.id) FILTER (WHERE pmk.status = 'Sakit') AS sakit,
               COUNT(e.id) FILTER (
                   WHERE pmk.status = 'Alpa' OR pmk.id IS NULL
               ) AS alpa
        FROM pertemuan_kuliah pk
        LEFT JOIN enrollments e
            ON e.jadwal_kuliah_id = pk.jadwal_kuliah_id
           AND e.status_approval::TEXT = 'Disetujui'
        LEFT JOIN presensi_mahasiswa_kuliah pmk
            ON pmk.pertemuan_id = pk.id AND pmk.enrollment_id = e.id
        WHERE pk.jadwal_kuliah_id = $1
        GROUP BY pk.id ORDER BY pk.pertemuan_ke
        "#,
    )
    .bind(jadwal_id)
    .fetch_all(pool)
    .await?)
}
