use super::{
    access,
    model::{
        AsesmenDetailResponse, AsesmenListRow, AsesmenMahasiswaRow, AsesmenRecord, DokumenAsesmen,
        FinishAsesmenPayload, JadwalAsesmenOption, NilaiAsesmenPayload, PelaksanaanAsesmen,
        PenggandaanAsesmen, PenggandaanPayload, PresensiAsesmenPayload, ReviewAsesmen,
        ReviewPayload, RosterAsesmen, SesiAsesmenResponse, UpsertAsesmenPayload,
    },
};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use uuid::Uuid;

const ASESMEN_SELECT: &str = r#"
    SELECT id, jadwal_kuliah_id, jenis::TEXT AS jenis, judul,
           mode::TEXT AS mode, bobot, durasi_menit, mulai_terjadwal,
           selesai_terjadwal, online_url, instruksi, sifat_ujian,
           hitung_sebagai_pertemuan, status::TEXT AS status, catatan_review
    FROM asesmen_kuliah
"#;

pub async fn schedule_options(
    pool: &DbPool,
    claims: &TokenClaims,
    tahun_akademik_id: Uuid,
) -> Result<Vec<JadwalAsesmenOption>, AppError> {
    let super_admin = access::has_role(claims, "SUPER_ADMIN");
    let academic = super_admin || access::has_role(claims, "STAF_AKADEMIK");
    let dosen_id = access::dosen_id(pool, claims.sub).await?;
    Ok(sqlx::query_as::<_, JadwalAsesmenOption>(
        r#"
        SELECT jk.id, mk.kode_mk, mk.nama_mk, jk.kelas, p.nama_prodi,
               ($1 OR EXISTS(
                   SELECT 1 FROM jadwal_dosen_pengampu jdp
                   WHERE jdp.jadwal_kuliah_id = jk.id AND jdp.dosen_id = $2
                     AND jdp.peran::TEXT = 'Koordinator'
               )) AS can_create
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
        JOIN prodi p ON p.id = mk.prodi_id
        WHERE jk.tahun_akademik_id = $3 AND (
            $1 OR EXISTS(
                SELECT 1 FROM jadwal_dosen_pengampu own
                WHERE own.jadwal_kuliah_id = jk.id AND own.dosen_id = $2
            )
        )
        ORDER BY mk.kode_mk, jk.kelas
        "#,
    )
    .bind(academic)
    .bind(dosen_id)
    .bind(tahun_akademik_id)
    .fetch_all(pool)
    .await?)
}

pub async fn list(
    pool: &DbPool,
    claims: &TokenClaims,
    tahun_akademik_id: Uuid,
) -> Result<Vec<AsesmenListRow>, AppError> {
    let super_admin = access::has_role(claims, "SUPER_ADMIN");
    let academic = super_admin || access::has_role(claims, "STAF_AKADEMIK");
    let production = super_admin || access::has_role(claims, "STAF_BAUM");
    let is_kaprodi = access::has_role(claims, "KAPRODI");
    let prodi_ids = if is_kaprodi {
        access::kaprodi_prodi_ids(pool, claims.sub).await?
    } else {
        Vec::new()
    };
    let dosen_id = access::dosen_id(pool, claims.sub).await?;

    Ok(sqlx::query_as::<_, AsesmenListRow>(
        r#"
        SELECT a.id, a.jadwal_kuliah_id, mk.kode_mk, mk.nama_mk, jk.kelas,
               p.nama_prodi, a.jenis::TEXT AS jenis, a.judul,
               a.mode::TEXT AS mode, a.bobot, a.durasi_menit,
               a.mulai_terjadwal, a.selesai_terjadwal, a.status::TEXT AS status,
               COUNT(DISTINCT da.id) AS jumlah_dokumen,
               MAX(pa.status) AS status_penggandaan,
               ($1 OR EXISTS(
                   SELECT 1 FROM jadwal_dosen_pengampu edit_jdp
                   WHERE edit_jdp.jadwal_kuliah_id = jk.id
                     AND edit_jdp.dosen_id = $5
                     AND edit_jdp.peran::TEXT = 'Koordinator'
               )) AS can_edit,
               ($1 OR ($2 AND mk.prodi_id = ANY($4))) AS can_review,
               ($3 AND a.mode = 'Manual') AS can_production,
               ($1 OR $6 OR EXISTS(
                   SELECT 1 FROM jadwal_dosen_pengampu exec_jdp
                   WHERE exec_jdp.jadwal_kuliah_id = jk.id AND exec_jdp.dosen_id = $5
               )) AS can_execute,
               ($1 OR EXISTS(
                   SELECT 1 FROM jadwal_dosen_pengampu grade_jdp
                   WHERE grade_jdp.jadwal_kuliah_id = jk.id
                     AND grade_jdp.dosen_id = $5
                     AND grade_jdp.peran::TEXT = 'Koordinator'
               )) AS can_grade
        FROM asesmen_kuliah a
        JOIN jadwal_kuliah jk ON jk.id = a.jadwal_kuliah_id
        JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
        JOIN prodi p ON p.id = mk.prodi_id
        LEFT JOIN dokumen_asesmen da ON da.asesmen_id = a.id
        LEFT JOIN penggandaan_asesmen pa ON pa.asesmen_id = a.id
        WHERE jk.tahun_akademik_id = $7 AND (
            $1 OR $6 OR (
                $3 AND a.mode = 'Manual'
                AND a.status IN ('Disetujui', 'SiapDilaksanakan', 'Berlangsung', 'Selesai', 'Dinilai', 'Dikunci')
            ) OR ($2 AND mk.prodi_id = ANY($4)) OR EXISTS(
                SELECT 1 FROM jadwal_dosen_pengampu own_jdp
                WHERE own_jdp.jadwal_kuliah_id = jk.id AND own_jdp.dosen_id = $5
            )
        )
        GROUP BY a.id, jk.id, mk.id, p.nama_prodi
        ORDER BY a.mulai_terjadwal, mk.kode_mk, jk.kelas
        "#,
    )
    .bind(super_admin)
    .bind(is_kaprodi)
    .bind(production)
    .bind(&prodi_ids)
    .bind(dosen_id)
    .bind(academic)
    .bind(tahun_akademik_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_record(pool: &DbPool, id: Uuid) -> Result<AsesmenRecord, AppError> {
    let query = format!("{} WHERE id = $1", ASESMEN_SELECT);
    Ok(sqlx::query_as::<_, AsesmenRecord>(&query)
        .bind(id)
        .fetch_one(pool)
        .await?)
}

pub async fn detail(pool: &DbPool, id: Uuid) -> Result<AsesmenDetailResponse, AppError> {
    let asesmen = get_record(pool, id).await?;
    let dokumen = sqlx::query_as::<_, DokumenAsesmen>(
        r#"
        SELECT id, jenis::TEXT AS jenis, versi, nama_file_asli, mime_type,
               ukuran_bytes, created_at
        FROM dokumen_asesmen WHERE asesmen_id = $1
        ORDER BY jenis, versi DESC
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    let review = sqlx::query_as::<_, ReviewAsesmen>(
        r#"
        SELECT ra.aksi, ra.catatan, u.full_name AS reviewer, ra.created_at
        FROM review_asesmen ra JOIN users u ON u.id = ra.reviewer_id
        WHERE ra.asesmen_id = $1 ORDER BY ra.created_at DESC
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    let penggandaan = sqlx::query_as::<_, PenggandaanAsesmen>(
        "SELECT jumlah_utama, jumlah_cadangan, status, catatan FROM penggandaan_asesmen WHERE asesmen_id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let pelaksanaan = sqlx::query_as::<_, PelaksanaanAsesmen>(
        r#"
        SELECT pa.mulai_aktual, pa.selesai_aktual, u.full_name AS pengawas,
               pa.versi_soal, pa.jumlah_lembar_diterima, pa.bap, pa.insiden
        FROM pelaksanaan_asesmen pa JOIN users u ON u.id = pa.pengawas_user_id
        WHERE pa.asesmen_id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let roster = sqlx::query_as::<_, RosterAsesmen>(
        r#"
        SELECT e.id AS enrollment_id, rm.nim, m.nama_mahasiswa,
               COALESCE(pa.status::TEXT, 'Alpa') AS status_presensi,
               pa.check_in_at, pa.sumber::TEXT AS sumber,
               latest.nilai, latest.attempt, latest.umpan_balik
        FROM enrollments e
        JOIN asesmen_kuliah a ON a.jadwal_kuliah_id = e.jadwal_kuliah_id
        JOIN registrasi_mahasiswa rm ON rm.id = e.registrasi_id
        JOIN mahasiswa m ON m.id = rm.mahasiswa_id
        LEFT JOIN presensi_asesmen pa
            ON pa.asesmen_id = a.id AND pa.enrollment_id = e.id
        LEFT JOIN LATERAL (
            SELECT na.nilai, na.attempt, na.umpan_balik
            FROM nilai_asesmen na
            WHERE na.asesmen_id = a.id AND na.enrollment_id = e.id
            ORDER BY na.attempt DESC LIMIT 1
        ) latest ON true
        WHERE a.id = $1 AND e.status_approval::TEXT = 'Disetujui'
        ORDER BY rm.nim
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    let sesi_presensi = sqlx::query_as::<_, SesiAsesmenResponse>(
        r#"
        SELECT kode, berlaku_sampai FROM sesi_presensi_asesmen
        WHERE asesmen_id = $1 AND aktif = true AND berlaku_sampai >= now()
        ORDER BY created_at DESC LIMIT 1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    let jumlah_peserta = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM enrollments e
        JOIN asesmen_kuliah a ON a.jadwal_kuliah_id = e.jadwal_kuliah_id
        WHERE a.id = $1 AND e.status_approval::TEXT = 'Disetujui'
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(AsesmenDetailResponse {
        asesmen,
        dokumen,
        review,
        penggandaan,
        pelaksanaan,
        roster,
        sesi_presensi,
        jumlah_peserta,
    })
}

pub async fn create(
    pool: &DbPool,
    user_id: Uuid,
    payload: UpsertAsesmenPayload,
) -> Result<Uuid, AppError> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO asesmen_kuliah (
            jadwal_kuliah_id, jenis, judul, mode, bobot, durasi_menit,
            mulai_terjadwal, selesai_terjadwal, online_url, instruksi,
            sifat_ujian, hitung_sebagai_pertemuan, dibuat_oleh
        ) VALUES (
            $1, $2::"JenisAsesmenKuliah", $3, $4::"ModeAsesmenKuliah", $5, $6,
            $7, $8, $9, $10, $11, $12, $13
        ) RETURNING id
        "#,
    )
    .bind(payload.jadwal_kuliah_id)
    .bind(payload.jenis)
    .bind(payload.judul)
    .bind(payload.mode)
    .bind(payload.bobot)
    .bind(payload.durasi_menit)
    .bind(payload.mulai_terjadwal)
    .bind(payload.selesai_terjadwal)
    .bind(payload.online_url)
    .bind(payload.instruksi)
    .bind(payload.sifat_ujian)
    .bind(payload.hitung_sebagai_pertemuan)
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

pub async fn update(
    pool: &DbPool,
    id: Uuid,
    payload: UpsertAsesmenPayload,
) -> Result<(), AppError> {
    let affected = sqlx::query(
        r#"
        UPDATE asesmen_kuliah SET
            jenis = $1::"JenisAsesmenKuliah", judul = $2,
            mode = $3::"ModeAsesmenKuliah", bobot = $4, durasi_menit = $5,
            mulai_terjadwal = $6, selesai_terjadwal = $7, online_url = $8,
            instruksi = $9, sifat_ujian = $10, hitung_sebagai_pertemuan = $11,
            updated_at = now()
        WHERE id = $12 AND status IN ('Draft', 'PerluRevisi')
        "#,
    )
    .bind(payload.jenis)
    .bind(payload.judul)
    .bind(payload.mode)
    .bind(payload.bobot)
    .bind(payload.durasi_menit)
    .bind(payload.mulai_terjadwal)
    .bind(payload.selesai_terjadwal)
    .bind(payload.online_url)
    .bind(payload.instruksi)
    .bind(payload.sifat_ujian)
    .bind(payload.hitung_sebagai_pertemuan)
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::BadRequest(
            "Asesmen tidak dapat diubah pada status saat ini.".to_string(),
        ));
    }
    Ok(())
}

pub async fn submit(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let affected = sqlx::query(
        r#"
        UPDATE asesmen_kuliah a SET status = 'Diajukan', diajukan_pada = now(),
            catatan_review = NULL, updated_at = now()
        WHERE a.id = $1 AND a.status IN ('Draft', 'PerluRevisi') AND (
            (a.mode = 'Online' AND NULLIF(TRIM(a.online_url), '') IS NOT NULL)
            OR (a.mode = 'Manual' AND EXISTS(
                SELECT 1 FROM dokumen_asesmen d WHERE d.asesmen_id = a.id AND d.jenis = 'Soal'
            ))
        )
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::BadRequest(
            "Lengkapi link ujian online atau unggah dokumen soal sebelum mengajukan.".to_string(),
        ));
    }
    Ok(())
}

pub async fn review(
    pool: &DbPool,
    id: Uuid,
    reviewer_id: Uuid,
    payload: ReviewPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let status = if payload.aksi == "Disetujui" {
        sqlx::query_scalar::<_, String>(
            "SELECT CASE WHEN mode = 'Online' THEN 'SiapDilaksanakan' ELSE 'Disetujui' END FROM asesmen_kuliah WHERE id = $1 AND status = 'Diajukan'",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::BadRequest("Asesmen tidak sedang diajukan.".to_string()))?
    } else if payload.aksi == "PerluRevisi" {
        "PerluRevisi".to_string()
    } else {
        return Err(AppError::BadRequest("Aksi review tidak valid.".to_string()));
    };
    let affected = sqlx::query(
        r#"
        UPDATE asesmen_kuliah SET status = $1::"StatusAsesmenKuliah",
            disetujui_oleh = CASE WHEN $1 = 'SiapDilaksanakan' OR $1 = 'Disetujui' THEN $2 ELSE NULL END,
            disetujui_pada = CASE WHEN $1 = 'SiapDilaksanakan' OR $1 = 'Disetujui' THEN now() ELSE NULL END,
            catatan_review = $3, updated_at = now()
        WHERE id = $4 AND status = 'Diajukan'
        "#,
    )
    .bind(&status)
    .bind(reviewer_id)
    .bind(&payload.catatan)
    .bind(id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::BadRequest(
            "Asesmen tidak sedang diajukan.".to_string(),
        ));
    }
    sqlx::query(
        "INSERT INTO review_asesmen (asesmen_id, aksi, catatan, reviewer_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(payload.aksi)
    .bind(payload.catatan)
    .bind(reviewer_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn add_document(
    pool: &DbPool,
    asesmen_id: Uuid,
    jenis: String,
    original_name: String,
    path: String,
    mime: String,
    size: i64,
    user_id: Uuid,
) -> Result<Uuid, AppError> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO dokumen_asesmen (
            asesmen_id, jenis, versi, nama_file_asli, path_file,
            mime_type, ukuran_bytes, diunggah_oleh
        ) VALUES (
            $1, $2::"JenisDokumenAsesmen",
            COALESCE((SELECT MAX(versi) + 1 FROM dokumen_asesmen WHERE asesmen_id = $1 AND jenis = $2::"JenisDokumenAsesmen"), 1),
            $3, $4, $5, $6, $7
        ) RETURNING id
        "#,
    )
    .bind(asesmen_id)
    .bind(jenis)
    .bind(original_name)
    .bind(path)
    .bind(mime)
    .bind(size)
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

pub async fn document_path(
    pool: &DbPool,
    asesmen_id: Uuid,
    document_id: Uuid,
) -> Result<(String, String, String), AppError> {
    sqlx::query_as::<_, (String, String, String)>(
        "SELECT path_file, nama_file_asli, jenis::TEXT FROM dokumen_asesmen WHERE id = $1 AND asesmen_id = $2",
    )
    .bind(document_id)
    .bind(asesmen_id)
    .fetch_optional(pool)
    .await?
    .ok_or(sqlx::Error::RowNotFound.into())
}

pub async fn audit_document_download(
    pool: &DbPool,
    document_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO audit_dokumen_asesmen (dokumen_id, user_id, aksi) VALUES ($1, $2, 'Download')",
    )
    .bind(document_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn production(
    pool: &DbPool,
    id: Uuid,
    user_id: Uuid,
    payload: PenggandaanPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let valid = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM asesmen_kuliah WHERE id = $1 AND mode = 'Manual' AND status IN ('Disetujui', 'SiapDilaksanakan'))",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if !valid {
        return Err(AppError::BadRequest(
            "Penggandaan hanya tersedia untuk ujian manual yang disetujui.".to_string(),
        ));
    }
    sqlx::query(
        r#"
        INSERT INTO penggandaan_asesmen (
            asesmen_id, jumlah_utama, jumlah_cadangan, status, catatan, diproses_oleh, diserahkan_pada
        ) VALUES ($1, $2, $3, $4, $5, $6, CASE WHEN $4 = 'Diserahkan' THEN now() ELSE NULL END)
        ON CONFLICT (asesmen_id) DO UPDATE SET
            jumlah_utama = EXCLUDED.jumlah_utama, jumlah_cadangan = EXCLUDED.jumlah_cadangan,
            status = EXCLUDED.status, catatan = EXCLUDED.catatan,
            diproses_oleh = EXCLUDED.diproses_oleh,
            diserahkan_pada = EXCLUDED.diserahkan_pada, updated_at = now()
        "#,
    )
    .bind(id)
    .bind(payload.jumlah_utama)
    .bind(payload.jumlah_cadangan)
    .bind(&payload.status)
    .bind(payload.catatan)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    if matches!(payload.status.as_str(), "Selesai" | "Diserahkan") {
        sqlx::query("UPDATE asesmen_kuliah SET status = 'SiapDilaksanakan', updated_at = now() WHERE id = $1 AND status = 'Disetujui'")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn start(
    pool: &DbPool,
    id: Uuid,
    user_id: Uuid,
    code: String,
) -> Result<SesiAsesmenResponse, AppError> {
    let mut tx = pool.begin().await?;
    let affected = sqlx::query(
        "UPDATE asesmen_kuliah SET status = 'Berlangsung', updated_at = now() WHERE id = $1 AND status = 'SiapDilaksanakan'",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::BadRequest(
            "Asesmen belum siap dilaksanakan.".to_string(),
        ));
    }
    let meeting_data = sqlx::query_as::<_, (Uuid, bool, String, String, time::OffsetDateTime)>(
        "SELECT jadwal_kuliah_id, hitung_sebagai_pertemuan, jenis::TEXT, judul, mulai_terjadwal FROM asesmen_kuliah WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if meeting_data.1 {
        let pertemuan_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO pertemuan_kuliah (
                jadwal_kuliah_id, pertemuan_ke, tanggal, topik_rencana,
                metode_pembelajaran, status, dibuka_oleh, dibuka_pada
            ) VALUES (
                $1,
                (SELECT COALESCE(MAX(pertemuan_ke), 0) + 1 FROM pertemuan_kuliah WHERE jadwal_kuliah_id = $1),
                ($2 AT TIME ZONE 'Asia/Jakarta')::DATE,
                $3, 'Asesmen', 'Dibuka', $4, now()
            ) RETURNING id
            "#,
        )
        .bind(meeting_data.0)
        .bind(meeting_data.4)
        .bind(format!("{}: {}", meeting_data.2, meeting_data.3))
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query("UPDATE asesmen_kuliah SET pertemuan_kuliah_id = $1 WHERE id = $2")
            .bind(pertemuan_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if let Some(dosen_id) = access::dosen_id(pool, user_id).await? {
            sqlx::query(
                r#"
                INSERT INTO presensi_dosen_kuliah (
                    pertemuan_id, dosen_id, status, check_in_at, sumber, dicatat_oleh
                ) VALUES ($1, $2, 'Hadir', now(), 'Sistem', $3)
                ON CONFLICT (pertemuan_id, dosen_id) DO NOTHING
                "#,
            )
            .bind(pertemuan_id)
            .bind(dosen_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        }
    }
    sqlx::query(
        "INSERT INTO pelaksanaan_asesmen (asesmen_id, pengawas_user_id) VALUES ($1, $2) ON CONFLICT (asesmen_id) DO UPDATE SET pengawas_user_id = EXCLUDED.pengawas_user_id, mulai_aktual = now(), updated_at = now()",
    )
    .bind(id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE sesi_presensi_asesmen SET aktif = false WHERE asesmen_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let berlaku_sampai = sqlx::query_scalar::<_, time::OffsetDateTime>(
        "INSERT INTO sesi_presensi_asesmen (asesmen_id, kode, berlaku_sampai, dibuat_oleh) VALUES ($1, $2, now() + interval '30 minutes', $3) RETURNING berlaku_sampai",
    )
    .bind(id)
    .bind(&code)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(SesiAsesmenResponse {
        kode: code,
        berlaku_sampai,
    })
}

pub async fn finish(
    pool: &DbPool,
    id: Uuid,
    payload: FinishAsesmenPayload,
) -> Result<(), AppError> {
    if payload.bap.trim().is_empty() {
        return Err(AppError::BadRequest("BAP ujian wajib diisi.".to_string()));
    }
    let mut tx = pool.begin().await?;
    let affected = sqlx::query(
        "UPDATE asesmen_kuliah SET status = 'Selesai', updated_at = now() WHERE id = $1 AND status = 'Berlangsung'",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::BadRequest(
            "Asesmen tidak sedang berlangsung.".to_string(),
        ));
    }
    sqlx::query(
        "UPDATE pelaksanaan_asesmen SET selesai_aktual = now(), versi_soal = $1, jumlah_lembar_diterima = $2, bap = $3, insiden = $4, updated_at = now() WHERE asesmen_id = $5",
    )
    .bind(payload.versi_soal.as_deref())
    .bind(payload.jumlah_lembar_diterima)
    .bind(&payload.bap)
    .bind(payload.insiden.as_deref())
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE sesi_presensi_asesmen SET aktif = false WHERE asesmen_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let pertemuan_id = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT pertemuan_kuliah_id FROM asesmen_kuliah WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if let Some(pertemuan_id) = pertemuan_id {
        sqlx::query(
            r#"
            UPDATE pertemuan_kuliah SET status = 'Ditutup', topik_realisasi = topik_rencana,
                bap = $1, ditutup_pada = now(), updated_at = now()
            WHERE id = $2
            "#,
        )
        .bind(&payload.bap)
        .bind(pertemuan_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO presensi_mahasiswa_kuliah (
                pertemuan_id, enrollment_id, status, check_in_at, sumber, catatan, dicatat_oleh
            )
            SELECT $1, e.id, COALESCE(pa.status, 'Alpa'), pa.check_in_at,
                   COALESCE(pa.sumber, 'Sistem'), pa.catatan, pa.dicatat_oleh
            FROM asesmen_kuliah a
            JOIN enrollments e ON e.jadwal_kuliah_id = a.jadwal_kuliah_id
            LEFT JOIN presensi_asesmen pa ON pa.asesmen_id = a.id AND pa.enrollment_id = e.id
            WHERE a.id = $2 AND e.status_approval::TEXT = 'Disetujui'
            ON CONFLICT (pertemuan_id, enrollment_id) DO UPDATE SET
                status = EXCLUDED.status, check_in_at = EXCLUDED.check_in_at,
                sumber = EXCLUDED.sumber, catatan = EXCLUDED.catatan,
                dicatat_oleh = EXCLUDED.dicatat_oleh, updated_at = now()
            "#,
        )
        .bind(pertemuan_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE presensi_dosen_kuliah SET check_out_at = now(), updated_at = now() WHERE pertemuan_id = $1",
        )
        .bind(pertemuan_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn manual_attendance(
    pool: &DbPool,
    id: Uuid,
    enrollment_id: Uuid,
    user_id: Uuid,
    payload: PresensiAsesmenPayload,
) -> Result<(), AppError> {
    let valid = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM asesmen_kuliah a
            JOIN enrollments e ON e.jadwal_kuliah_id = a.jadwal_kuliah_id
            WHERE a.id = $1 AND e.id = $2 AND a.status = 'Berlangsung'
              AND e.status_approval::TEXT = 'Disetujui'
        )
        "#,
    )
    .bind(id)
    .bind(enrollment_id)
    .fetch_one(pool)
    .await?;
    if !valid {
        return Err(AppError::BadRequest(
            "Presensi hanya dapat diubah saat ujian berlangsung untuk peserta terdaftar."
                .to_string(),
        ));
    }
    sqlx::query(
        r#"
        INSERT INTO presensi_asesmen (
            asesmen_id, enrollment_id, status, check_in_at, sumber, catatan, dicatat_oleh
        ) VALUES (
            $1, $2, $3::"StatusPresensiMahasiswa",
            CASE WHEN $3 IN ('Hadir', 'Terlambat') THEN now() ELSE NULL END,
            'ManualDosen', $4, $5
        )
        ON CONFLICT (asesmen_id, enrollment_id) DO UPDATE SET
            status = EXCLUDED.status, check_in_at = EXCLUDED.check_in_at,
            sumber = EXCLUDED.sumber, catatan = EXCLUDED.catatan,
            dicatat_oleh = EXCLUDED.dicatat_oleh, updated_at = now()
        "#,
    )
    .bind(id)
    .bind(enrollment_id)
    .bind(payload.status)
    .bind(payload.catatan)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn grade(
    pool: &DbPool,
    id: Uuid,
    enrollment_id: Uuid,
    user_id: Uuid,
    payload: NilaiAsesmenPayload,
) -> Result<(), AppError> {
    let attempt = payload.attempt.unwrap_or(1);
    let mut tx = pool.begin().await?;
    let allowed = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM asesmen_kuliah WHERE id = $1 AND status IN ('Selesai', 'Dinilai'))",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if !allowed {
        return Err(AppError::BadRequest(
            "Nilai hanya dapat diisi setelah ujian selesai.".to_string(),
        ));
    }
    let valid_enrollment = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM asesmen_kuliah a
            JOIN enrollments e ON e.jadwal_kuliah_id = a.jadwal_kuliah_id
            WHERE a.id = $1 AND e.id = $2 AND e.status_approval::TEXT = 'Disetujui'
        )
        "#,
    )
    .bind(id)
    .bind(enrollment_id)
    .fetch_one(&mut *tx)
    .await?;
    if !valid_enrollment {
        return Err(AppError::BadRequest(
            "Mahasiswa bukan peserta ujian ini.".to_string(),
        ));
    }
    sqlx::query(
        r#"
        INSERT INTO nilai_asesmen (
            asesmen_id, enrollment_id, attempt, nilai, umpan_balik, dinilai_oleh
        ) VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (asesmen_id, enrollment_id, attempt) DO UPDATE SET
            nilai = EXCLUDED.nilai, umpan_balik = EXCLUDED.umpan_balik,
            dinilai_oleh = EXCLUDED.dinilai_oleh, updated_at = now()
        "#,
    )
    .bind(id)
    .bind(enrollment_id)
    .bind(attempt)
    .bind(payload.nilai)
    .bind(payload.umpan_balik)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE asesmen_kuliah SET status = 'Dinilai', updated_at = now() WHERE id = $1 AND status = 'Selesai'")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn lock(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let affected = sqlx::query(
        r#"
        UPDATE asesmen_kuliah a SET status = 'Dikunci', updated_at = now()
        WHERE a.id = $1 AND a.status = 'Dinilai' AND NOT EXISTS(
            SELECT 1 FROM enrollments e
            WHERE e.jadwal_kuliah_id = a.jadwal_kuliah_id
              AND e.status_approval::TEXT = 'Disetujui'
              AND NOT EXISTS(
                  SELECT 1 FROM nilai_asesmen n
                  WHERE n.asesmen_id = a.id AND n.enrollment_id = e.id
              )
        )
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::BadRequest(
            "Semua mahasiswa harus memiliki nilai sebelum asesmen dikunci.".to_string(),
        ));
    }
    Ok(())
}

pub async fn reopen_grade(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let affected = sqlx::query(
        r#"
        UPDATE asesmen_kuliah a SET status = 'Dinilai', updated_at = now()
        WHERE a.id = $1 AND a.status = 'Dikunci'
          AND COALESCE((
              SELECT n.status::TEXT FROM nilai_akhir_kuliah n
              WHERE n.jadwal_kuliah_id = a.jadwal_kuliah_id
          ), 'Draft') IN ('Draft', 'PerluRevisi')
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::BadRequest(
            "Nilai tidak dapat dibuka saat rekap sedang direview, disetujui, atau sudah dipublikasikan."
                .to_string(),
        ));
    }
    Ok(())
}

pub async fn student_list(
    pool: &DbPool,
    user_id: Uuid,
    tahun_akademik_id: Uuid,
) -> Result<Vec<AsesmenMahasiswaRow>, AppError> {
    Ok(sqlx::query_as::<_, AsesmenMahasiswaRow>(
        r#"
        SELECT a.id, mk.kode_mk, mk.nama_mk, jk.kelas,
               a.jenis::TEXT AS jenis, a.judul, a.mode::TEXT AS mode,
               a.mulai_terjadwal, a.selesai_terjadwal, a.durasi_menit,
               a.instruksi, a.status::TEXT AS status,
               CASE WHEN a.mode = 'Online' AND a.status = 'Berlangsung'
                    THEN a.online_url ELSE NULL END AS online_url,
               pa.status::TEXT AS status_presensi,
               CASE WHEN a.status = 'Dikunci' THEN latest.nilai ELSE NULL END AS nilai
        FROM enrollments e
        JOIN registrasi_mahasiswa rm ON rm.id = e.registrasi_id
        JOIN mahasiswa m ON m.id = rm.mahasiswa_id
        JOIN jadwal_kuliah jk ON jk.id = e.jadwal_kuliah_id
        JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
        JOIN asesmen_kuliah a ON a.jadwal_kuliah_id = jk.id
        LEFT JOIN presensi_asesmen pa ON pa.asesmen_id = a.id AND pa.enrollment_id = e.id
        LEFT JOIN LATERAL (
            SELECT nilai FROM nilai_asesmen n
            WHERE n.asesmen_id = a.id AND n.enrollment_id = e.id
            ORDER BY attempt DESC LIMIT 1
        ) latest ON true
        WHERE m.user_id = $1 AND e.tahun_akademik_id = $2
          AND e.status_approval::TEXT = 'Disetujui'
          AND a.status NOT IN ('Draft', 'Diajukan', 'PerluRevisi', 'Dibatalkan')
        ORDER BY a.mulai_terjadwal
        "#,
    )
    .bind(user_id)
    .bind(tahun_akademik_id)
    .fetch_all(pool)
    .await?)
}

pub async fn student_check_in(pool: &DbPool, user_id: Uuid, code: String) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let asesmen_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT spa.asesmen_id FROM sesi_presensi_asesmen spa
        JOIN asesmen_kuliah a ON a.id = spa.asesmen_id
        WHERE UPPER(spa.kode) = UPPER($1) AND spa.aktif = true
          AND spa.berlaku_sampai >= now() AND a.status = 'Berlangsung'
        ORDER BY spa.created_at DESC LIMIT 1
        "#,
    )
    .bind(code.trim())
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::BadRequest("Kode ujian tidak valid atau kedaluwarsa.".to_string()))?;
    let enrollment_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT e.id FROM enrollments e
        JOIN registrasi_mahasiswa rm ON rm.id = e.registrasi_id
        JOIN mahasiswa m ON m.id = rm.mahasiswa_id
        JOIN asesmen_kuliah a ON a.jadwal_kuliah_id = e.jadwal_kuliah_id
        WHERE a.id = $1 AND m.user_id = $2 AND e.status_approval::TEXT = 'Disetujui'
        "#,
    )
    .bind(asesmen_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::Forbidden("Anda tidak terdaftar pada ujian ini.".to_string()))?;
    sqlx::query(
        r#"
        INSERT INTO presensi_asesmen (
            asesmen_id, enrollment_id, status, check_in_at, sumber, dicatat_oleh
        ) VALUES ($1, $2, 'Hadir', now(), 'KodeDinamis', $3)
        ON CONFLICT (asesmen_id, enrollment_id) DO UPDATE SET
            status = 'Hadir', check_in_at = now(), sumber = 'KodeDinamis',
            dicatat_oleh = EXCLUDED.dicatat_oleh, updated_at = now()
        WHERE presensi_asesmen.status = 'Alpa'
        "#,
    )
    .bind(asesmen_id)
    .bind(enrollment_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}
