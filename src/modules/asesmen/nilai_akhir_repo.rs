use super::{
    access,
    model::{
        AsesmenQuery, KelasNilaiAkhir, KomponenNilaiAkhir, MahasiswaNilaiAkhir, NilaiAkhirDetail,
        NilaiKomponenMahasiswa, RiwayatNilaiAkhir, SkalaNilaiRow, UpsertSkalaNilaiPayload,
    },
};
use crate::{db::DbPool, errors::AppError, modules::auth::middleware::TokenClaims};
use rust_decimal::Decimal;
use sqlx::{FromRow, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct StudentBase {
    enrollment_id: Uuid,
    nim: String,
    nama_mahasiswa: String,
}

#[derive(Debug, FromRow)]
struct StudentScore {
    enrollment_id: Uuid,
    asesmen_id: Uuid,
    nilai: Decimal,
}

#[derive(Debug, FromRow)]
struct ScaleRow {
    nilai_huruf: String,
    nilai_indeks: Decimal,
    bobot_minimum: Decimal,
    bobot_maksimum: Decimal,
}

struct Calculation {
    components: Vec<KomponenNilaiAkhir>,
    students: Vec<MahasiswaNilaiAkhir>,
    scales_available: bool,
}

pub async fn list_classes(
    pool: &DbPool,
    claims: &TokenClaims,
    query: AsesmenQuery,
) -> Result<Vec<KelasNilaiAkhir>, AppError> {
    let super_admin = access::has_role(claims, "SUPER_ADMIN");
    let academic = super_admin || access::has_role(claims, "STAF_AKADEMIK");
    let is_kaprodi = super_admin || access::has_role(claims, "KAPRODI");
    let prodi_ids = if is_kaprodi && !super_admin {
        access::kaprodi_prodi_ids(pool, claims.sub).await?
    } else {
        Vec::new()
    };
    let dosen_id = access::dosen_id(pool, claims.sub).await?;

    Ok(sqlx::query_as::<_, KelasNilaiAkhir>(
        r#"
        SELECT jk.id AS jadwal_kuliah_id, mk.prodi_id, mk.kode_mk, mk.nama_mk, jk.kelas,
               p.nama_prodi, COALESCE(na.status::TEXT, 'Draft') AS status,
               COALESCE(SUM(a.bobot), 0) AS total_bobot,
               COUNT(a.id) AS jumlah_asesmen,
               COUNT(a.id) FILTER (WHERE a.status::TEXT = 'Dikunci') AS jumlah_asesmen_dikunci,
               ($1 OR EXISTS(
                    SELECT 1 FROM jadwal_dosen_pengampu c
                    WHERE c.jadwal_kuliah_id = jk.id AND c.dosen_id = $3
                      AND c.peran::TEXT = 'Koordinator'
               )) AS can_submit,
               ($1 OR ($4 AND mk.prodi_id = ANY($5))) AS can_review,
               $2 AS can_publish
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
        JOIN prodi p ON p.id = mk.prodi_id
        LEFT JOIN asesmen_kuliah a ON a.jadwal_kuliah_id = jk.id
        LEFT JOIN nilai_akhir_kuliah na ON na.jadwal_kuliah_id = jk.id
        WHERE jk.tahun_akademik_id = $6 AND (
            $2 OR ($4 AND mk.prodi_id = ANY($5)) OR EXISTS(
                SELECT 1 FROM jadwal_dosen_pengampu own
                WHERE own.jadwal_kuliah_id = jk.id AND own.dosen_id = $3
            )
        )
        GROUP BY jk.id, mk.id, p.nama_prodi, na.status
        ORDER BY mk.kode_mk, jk.kelas
        "#,
    )
    .bind(super_admin)
    .bind(academic)
    .bind(dosen_id)
    .bind(is_kaprodi)
    .bind(&prodi_ids)
    .bind(query.tahun_akademik_id)
    .fetch_all(pool)
    .await?)
}

async fn class_header(
    pool: &DbPool,
    jadwal_id: Uuid,
    permission: &access::AsesmenAccess,
) -> Result<KelasNilaiAkhir, AppError> {
    Ok(sqlx::query_as::<_, KelasNilaiAkhir>(
        r#"
        SELECT jk.id AS jadwal_kuliah_id, mk.prodi_id, mk.kode_mk, mk.nama_mk, jk.kelas,
               p.nama_prodi, COALESCE(na.status::TEXT, 'Draft') AS status,
               COALESCE(SUM(a.bobot), 0) AS total_bobot,
               COUNT(a.id) AS jumlah_asesmen,
               COUNT(a.id) FILTER (WHERE a.status::TEXT = 'Dikunci') AS jumlah_asesmen_dikunci,
               $2 AS can_submit, $3 AS can_review, $4 AS can_publish
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
        JOIN prodi p ON p.id = mk.prodi_id
        LEFT JOIN asesmen_kuliah a ON a.jadwal_kuliah_id = jk.id
        LEFT JOIN nilai_akhir_kuliah na ON na.jadwal_kuliah_id = jk.id
        WHERE jk.id = $1
        GROUP BY jk.id, mk.id, p.nama_prodi, na.status
        "#,
    )
    .bind(jadwal_id)
    .bind(permission.coordinator)
    .bind(permission.kaprodi)
    .bind(permission.academic)
    .fetch_optional(pool)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?)
}

async fn calculate(pool: &DbPool, jadwal_id: Uuid) -> Result<Calculation, AppError> {
    let components = sqlx::query_as::<_, KomponenNilaiAkhir>(
        r#"
        SELECT id, jenis::TEXT AS jenis, judul, bobot, status::TEXT AS status
        FROM asesmen_kuliah
        WHERE jadwal_kuliah_id = $1
        ORDER BY mulai_terjadwal, jenis::TEXT, judul
        "#,
    )
    .bind(jadwal_id)
    .fetch_all(pool)
    .await?;
    let bases = sqlx::query_as::<_, StudentBase>(
        r#"
        SELECT e.id AS enrollment_id, rm.nim, m.nama_mahasiswa
        FROM enrollments e
        JOIN registrasi_mahasiswa rm ON rm.id = e.registrasi_id
        JOIN mahasiswa m ON m.id = rm.mahasiswa_id
        WHERE e.jadwal_kuliah_id = $1 AND e.status_approval::TEXT = 'Disetujui'
        ORDER BY rm.nim
        "#,
    )
    .bind(jadwal_id)
    .fetch_all(pool)
    .await?;
    let scores = sqlx::query_as::<_, StudentScore>(
        r#"
        SELECT DISTINCT ON (n.enrollment_id, n.asesmen_id)
               n.enrollment_id, n.asesmen_id, n.nilai
        FROM nilai_asesmen n
        JOIN asesmen_kuliah a ON a.id = n.asesmen_id
        WHERE a.jadwal_kuliah_id = $1
        ORDER BY n.enrollment_id, n.asesmen_id, n.attempt DESC
        "#,
    )
    .bind(jadwal_id)
    .fetch_all(pool)
    .await?;
    let scales = sqlx::query_as::<_, ScaleRow>(
        r#"
        SELECT sn.nilai_huruf, sn.nilai_indeks, sn.bobot_minimum, sn.bobot_maksimum
        FROM skala_nilai sn
        JOIN mata_kuliah mk ON mk.prodi_id = sn.prodi_id
        JOIN jadwal_kuliah jk ON jk.matakuliah_id = mk.id
        JOIN tahun_akademik ta ON ta.id = jk.tahun_akademik_id
        WHERE jk.id = $1
          AND sn.tanggal_mulai_efektif <= ta.tanggal_selesai
          AND (sn.tanggal_akhir_efektif IS NULL OR sn.tanggal_akhir_efektif >= ta.tanggal_mulai)
        ORDER BY sn.tanggal_mulai_efektif DESC, sn.bobot_minimum DESC
        "#,
    )
    .bind(jadwal_id)
    .fetch_all(pool)
    .await?;

    let score_map: HashMap<(Uuid, Uuid), Decimal> = scores
        .into_iter()
        .map(|score| ((score.enrollment_id, score.asesmen_id), score.nilai))
        .collect();
    let all_locked =
        !components.is_empty() && components.iter().all(|item| item.status == "Dikunci");
    let mut students = Vec::with_capacity(bases.len());
    for base in bases {
        let component_scores: Vec<NilaiKomponenMahasiswa> = components
            .iter()
            .map(|component| NilaiKomponenMahasiswa {
                asesmen_id: component.id,
                nilai: score_map.get(&(base.enrollment_id, component.id)).copied(),
            })
            .collect();
        let complete = all_locked && component_scores.iter().all(|item| item.nilai.is_some());
        let final_score = complete.then(|| {
            component_scores
                .iter()
                .zip(&components)
                .map(|(score, component)| score.nilai.unwrap_or_default() * component.bobot)
                .sum::<Decimal>()
                .checked_div(Decimal::from(100))
                .unwrap_or_default()
                .round_dp(2)
        });
        let scale = final_score.and_then(|score| {
            scales
                .iter()
                .find(|scale| score >= scale.bobot_minimum && score <= scale.bobot_maksimum)
        });
        students.push(MahasiswaNilaiAkhir {
            enrollment_id: base.enrollment_id,
            nim: base.nim,
            nama_mahasiswa: base.nama_mahasiswa,
            komponen: component_scores,
            lengkap: complete,
            nilai_akhir: final_score,
            nilai_huruf: scale.map(|item| item.nilai_huruf.clone()),
            nilai_indeks: scale.map(|item| item.nilai_indeks),
        });
    }
    Ok(Calculation {
        components,
        students,
        scales_available: !scales.is_empty(),
    })
}

pub async fn detail(
    pool: &DbPool,
    claims: &TokenClaims,
    jadwal_id: Uuid,
) -> Result<NilaiAkhirDetail, AppError> {
    let permission = access::for_jadwal(pool, claims, jadwal_id).await?;
    if !(permission.assigned || permission.kaprodi || permission.academic) {
        return Err(AppError::Forbidden(
            "Anda tidak memiliki akses ke rekap nilai kelas ini.".to_string(),
        ));
    }
    let kelas = class_header(pool, jadwal_id, &permission).await?;
    let calculation = calculate(pool, jadwal_id).await?;
    let history = sqlx::query_as::<_, RiwayatNilaiAkhir>(
        r#"
        SELECT r.aksi, r.catatan, u.full_name AS dilakukan_oleh, r.created_at
        FROM riwayat_nilai_akhir r
        JOIN nilai_akhir_kuliah n ON n.id = r.nilai_akhir_kuliah_id
        JOIN users u ON u.id = r.dilakukan_oleh
        WHERE n.jadwal_kuliah_id = $1 ORDER BY r.created_at DESC
        "#,
    )
    .bind(jadwal_id)
    .fetch_all(pool)
    .await?;
    Ok(NilaiAkhirDetail {
        kelas,
        komponen: calculation.components,
        mahasiswa: calculation.students,
        riwayat: history,
        skala_tersedia: calculation.scales_available,
    })
}

fn validate_calculation(calculation: &Calculation) -> Result<(), AppError> {
    if calculation.components.is_empty() {
        return Err(AppError::BadRequest(
            "Belum ada komponen asesmen pada kelas ini.".to_string(),
        ));
    }
    let total: Decimal = calculation.components.iter().map(|item| item.bobot).sum();
    if total != Decimal::from(100) {
        return Err(AppError::BadRequest(format!(
            "Total bobot asesmen harus tepat 100%. Saat ini: {total}%."
        )));
    }
    if calculation
        .components
        .iter()
        .any(|item| item.status != "Dikunci")
    {
        return Err(AppError::BadRequest(
            "Seluruh asesmen harus dinilai dan dikunci terlebih dahulu.".to_string(),
        ));
    }
    if calculation.students.is_empty() {
        return Err(AppError::BadRequest(
            "Kelas belum memiliki mahasiswa yang disetujui.".to_string(),
        ));
    }
    if calculation.students.iter().any(|item| !item.lengkap) {
        return Err(AppError::BadRequest(
            "Masih ada mahasiswa yang nilainya belum lengkap.".to_string(),
        ));
    }
    if !calculation.scales_available
        || calculation
            .students
            .iter()
            .any(|item| item.nilai_huruf.is_none())
    {
        return Err(AppError::BadRequest(
            "Skala nilai Prodi belum tersedia atau tidak mencakup seluruh rentang nilai."
                .to_string(),
        ));
    }
    Ok(())
}

async fn add_history(
    tx: &mut Transaction<'_, Postgres>,
    rekap_id: Uuid,
    action: &str,
    note: Option<&str>,
    user_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO riwayat_nilai_akhir (nilai_akhir_kuliah_id, aksi, catatan, dilakukan_oleh) VALUES ($1, $2, $3, $4)",
    )
    .bind(rekap_id)
    .bind(action)
    .bind(note)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn submit(pool: &DbPool, claims: &TokenClaims, jadwal_id: Uuid) -> Result<(), AppError> {
    let permission = access::for_jadwal(pool, claims, jadwal_id).await?;
    access::require_grade(&permission)?;
    let calculation = calculate(pool, jadwal_id).await?;
    validate_calculation(&calculation)?;
    let mut tx = pool.begin().await?;
    let rekap_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO nilai_akhir_kuliah (jadwal_kuliah_id, status, diajukan_oleh, diajukan_pada)
        VALUES ($1, 'Diajukan', $2, now())
        ON CONFLICT (jadwal_kuliah_id) DO UPDATE SET
            status = 'Diajukan', catatan = NULL, diajukan_oleh = EXCLUDED.diajukan_oleh,
            diajukan_pada = now(), disetujui_oleh = NULL, disetujui_pada = NULL,
            updated_at = now()
        WHERE nilai_akhir_kuliah.status IN ('Draft', 'PerluRevisi')
        RETURNING id
        "#,
    )
    .bind(jadwal_id)
    .bind(claims.sub)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::BadRequest("Nilai akhir tidak dapat diajukan pada status saat ini.".to_string())
    })?;
    add_history(&mut tx, rekap_id, "Diajukan", None, claims.sub).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn review(
    pool: &DbPool,
    claims: &TokenClaims,
    jadwal_id: Uuid,
    action: &str,
    note: Option<&str>,
) -> Result<(), AppError> {
    let permission = access::for_jadwal(pool, claims, jadwal_id).await?;
    if !permission.kaprodi {
        return Err(AppError::Forbidden(
            "Hanya Kaprodi terkait yang dapat memvalidasi nilai akhir.".to_string(),
        ));
    }
    if !["Disetujui", "PerluRevisi"].contains(&action) {
        return Err(AppError::BadRequest(
            "Keputusan review tidak valid.".to_string(),
        ));
    }
    let mut tx = pool.begin().await?;
    let rekap_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE nilai_akhir_kuliah SET status = $1::"StatusNilaiAkhir", catatan = $2,
            disetujui_oleh = CASE WHEN $1 = 'Disetujui' THEN $3 ELSE NULL END,
            disetujui_pada = CASE WHEN $1 = 'Disetujui' THEN now() ELSE NULL END,
            updated_at = now()
        WHERE jadwal_kuliah_id = $4 AND status = 'Diajukan' RETURNING id
        "#,
    )
    .bind(action)
    .bind(note)
    .bind(claims.sub)
    .bind(jadwal_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::BadRequest("Nilai akhir tidak sedang diajukan.".to_string()))?;
    add_history(&mut tx, rekap_id, action, note, claims.sub).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn publish(pool: &DbPool, claims: &TokenClaims, jadwal_id: Uuid) -> Result<(), AppError> {
    let permission = access::for_jadwal(pool, claims, jadwal_id).await?;
    if !permission.academic {
        return Err(AppError::Forbidden(
            "Hanya staf akademik yang dapat mempublikasikan nilai akhir.".to_string(),
        ));
    }
    let calculation = calculate(pool, jadwal_id).await?;
    validate_calculation(&calculation)?;
    let mut tx = pool.begin().await?;
    let status = sqlx::query_scalar::<_, String>(
        "SELECT status::TEXT FROM nilai_akhir_kuliah WHERE jadwal_kuliah_id = $1 FOR UPDATE",
    )
    .bind(jadwal_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::BadRequest("Nilai akhir belum diajukan.".to_string()))?;
    if status != "Disetujui" {
        return Err(AppError::BadRequest(
            "Nilai akhir harus disetujui Kaprodi sebelum dipublikasikan.".to_string(),
        ));
    }
    for student in &calculation.students {
        sqlx::query(
            "UPDATE enrollments SET nilai_angka = $1, nilai_huruf = $2, nilai_indeks = $3, updated_at = now() WHERE id = $4",
        )
        .bind(student.nilai_akhir)
        .bind(&student.nilai_huruf)
        .bind(student.nilai_indeks)
        .bind(student.enrollment_id)
        .execute(&mut *tx)
        .await?;
    }
    let rekap_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        UPDATE nilai_akhir_kuliah SET status = 'Dipublikasikan', dipublikasikan_oleh = $1,
            dipublikasikan_pada = now(), updated_at = now()
        WHERE jadwal_kuliah_id = $2 RETURNING id
        "#,
    )
    .bind(claims.sub)
    .bind(jadwal_id)
    .fetch_one(&mut *tx)
    .await?;
    add_history(&mut tx, rekap_id, "Dipublikasikan", None, claims.sub).await?;
    tx.commit().await?;
    Ok(())
}

async fn require_scale_manager(
    pool: &DbPool,
    claims: &TokenClaims,
    prodi_id: Uuid,
) -> Result<(), AppError> {
    if access::has_role(claims, "SUPER_ADMIN")
        || (access::has_role(claims, "KAPRODI")
            && access::kaprodi_prodi_ids(pool, claims.sub)
                .await?
                .contains(&prodi_id))
    {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Hanya Kaprodi terkait atau Super Admin yang dapat mengatur skala nilai.".to_string(),
        ))
    }
}

pub async fn list_scales(
    pool: &DbPool,
    claims: &TokenClaims,
    prodi_id: Uuid,
) -> Result<Vec<SkalaNilaiRow>, AppError> {
    require_scale_manager(pool, claims, prodi_id).await?;
    Ok(sqlx::query_as::<_, SkalaNilaiRow>(
        r#"
        SELECT id, prodi_id, nilai_huruf, nilai_indeks, bobot_minimum, bobot_maksimum,
               to_char(tanggal_mulai_efektif, 'YYYY-MM-DD') AS tanggal_mulai_efektif,
               to_char(tanggal_akhir_efektif, 'YYYY-MM-DD') AS tanggal_akhir_efektif,
               (id_bobot_nilai_feeder IS NOT NULL) AS dari_feeder
        FROM skala_nilai WHERE prodi_id = $1
        ORDER BY tanggal_mulai_efektif DESC, bobot_minimum DESC
        "#,
    )
    .bind(prodi_id)
    .fetch_all(pool)
    .await?)
}

pub async fn save_scales(
    pool: &DbPool,
    claims: &TokenClaims,
    prodi_id: Uuid,
    payload: UpsertSkalaNilaiPayload,
) -> Result<Vec<SkalaNilaiRow>, AppError> {
    require_scale_manager(pool, claims, prodi_id).await?;
    if payload.items.is_empty() {
        return Err(AppError::BadRequest(
            "Skala nilai tidak boleh kosong.".to_string(),
        ));
    }
    let mut parsed = Vec::with_capacity(payload.items.len());
    for item in payload.items {
        let letter = item.nilai_huruf.trim().to_uppercase();
        if letter.is_empty()
            || item.bobot_minimum < Decimal::ZERO
            || item.bobot_maksimum > Decimal::from(100)
            || item.bobot_minimum > item.bobot_maksimum
            || item.nilai_indeks < Decimal::ZERO
            || item.nilai_indeks > Decimal::from(4)
        {
            return Err(AppError::BadRequest(
                "Huruf, indeks, atau rentang skala nilai tidak valid.".to_string(),
            ));
        }
        let start = time::Date::parse(
            &item.tanggal_mulai_efektif,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )?;
        let end = item
            .tanggal_akhir_efektif
            .as_deref()
            .filter(|value| !value.is_empty())
            .map(|value| {
                time::Date::parse(
                    value,
                    &time::format_description::well_known::Iso8601::DEFAULT,
                )
            })
            .transpose()?;
        if end.is_some_and(|value| value < start) {
            return Err(AppError::BadRequest(
                "Tanggal akhir skala tidak boleh sebelum tanggal mulai.".to_string(),
            ));
        }
        parsed.push((item, letter, start, end));
    }
    for left in 0..parsed.len() {
        for right in (left + 1)..parsed.len() {
            let (a, _, a_start, _) = &parsed[left];
            let (b, _, b_start, _) = &parsed[right];
            if a_start == b_start
                && a.bobot_minimum <= b.bobot_maksimum
                && b.bobot_minimum <= a.bobot_maksimum
            {
                return Err(AppError::BadRequest(
                    "Rentang pada periode efektif yang sama tidak boleh tumpang tindih."
                        .to_string(),
                ));
            }
        }
    }

    let mut tx = pool.begin().await?;
    let old_local_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM skala_nilai WHERE prodi_id = $1 AND id_bobot_nilai_feeder IS NULL",
    )
    .bind(prodi_id)
    .fetch_all(&mut *tx)
    .await?;
    let mut retained_ids = Vec::new();
    for (item, letter, start, end) in parsed {
        let id = if let Some(id) = item.id {
            let affected = sqlx::query(
                r#"
                UPDATE skala_nilai SET nilai_huruf=$1, nilai_indeks=$2, bobot_minimum=$3,
                    bobot_maksimum=$4, tanggal_mulai_efektif=$5,
                    tanggal_akhir_efektif=$6, updated_at=now()
                WHERE id=$7 AND prodi_id=$8 AND id_bobot_nilai_feeder IS NULL
                "#,
            )
            .bind(letter)
            .bind(item.nilai_indeks)
            .bind(item.bobot_minimum)
            .bind(item.bobot_maksimum)
            .bind(start)
            .bind(end)
            .bind(id)
            .bind(prodi_id)
            .execute(&mut *tx)
            .await?
            .rows_affected();
            if affected == 0 {
                return Err(AppError::BadRequest(
                    "Skala Feeder tidak dapat diubah dari halaman ini.".to_string(),
                ));
            }
            id
        } else {
            sqlx::query_scalar::<_, Uuid>(
                r#"
                INSERT INTO skala_nilai (prodi_id, nilai_huruf, nilai_indeks, bobot_minimum,
                    bobot_maksimum, tanggal_mulai_efektif, tanggal_akhir_efektif)
                VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id
                "#,
            )
            .bind(prodi_id)
            .bind(letter)
            .bind(item.nilai_indeks)
            .bind(item.bobot_minimum)
            .bind(item.bobot_maksimum)
            .bind(start)
            .bind(end)
            .fetch_one(&mut *tx)
            .await?
        };
        retained_ids.push(id);
    }
    for id in old_local_ids {
        if !retained_ids.contains(&id) {
            sqlx::query("DELETE FROM skala_nilai WHERE id=$1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await?;
    list_scales(pool, claims, prodi_id).await
}
