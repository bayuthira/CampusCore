use super::model::*;
use crate::modules::{asesmen::access, auth::middleware::TokenClaims};
use crate::{db::DbPool, errors::AppError};
use rust_decimal::Decimal;
use uuid::Uuid;

pub async fn status(pool: &DbPool, tahun_id: Uuid) -> Result<StatusAkhirSemester, AppError> {
    Ok(sqlx::query_as::<_, StatusAkhirSemester>(
        r#"
        SELECT ta.id AS tahun_akademik_id, ta.nama, ta.status_penutupan,
               COUNT(DISTINCT jk.id) FILTER (WHERE EXISTS(
                   SELECT 1 FROM enrollments e WHERE e.jadwal_kuliah_id=jk.id
                     AND e.status_approval::TEXT='Disetujui'
               )) AS jumlah_kelas,
               COUNT(DISTINCT jk.id) FILTER (WHERE EXISTS(
                   SELECT 1 FROM enrollments e WHERE e.jadwal_kuliah_id=jk.id
                     AND e.status_approval::TEXT='Disetujui'
               ) AND na.status::TEXT='Dipublikasikan') AS kelas_siap,
               COUNT(DISTINCT jk.id) FILTER (WHERE EXISTS(
                   SELECT 1 FROM enrollments e WHERE e.jadwal_kuliah_id=jk.id
                     AND e.status_approval::TEXT='Disetujui'
               ) AND COALESCE(na.status::TEXT,'Draft')<>'Dipublikasikan') AS kelas_belum_siap,
               COUNT(DISTINCT e.registrasi_id) FILTER (WHERE e.status_approval::TEXT='Disetujui') AS jumlah_mahasiswa,
               COUNT(e.id) FILTER (WHERE e.nilai_angka IS NOT NULL) AS jumlah_nilai,
               u.full_name AS ditutup_oleh, ta.ditutup_pada
        FROM tahun_akademik ta
        LEFT JOIN jadwal_kuliah jk ON jk.tahun_akademik_id=ta.id
        LEFT JOIN nilai_akhir_kuliah na ON na.jadwal_kuliah_id=jk.id
        LEFT JOIN enrollments e ON e.jadwal_kuliah_id=jk.id
        LEFT JOIN users u ON u.id=ta.ditutup_oleh
        WHERE ta.id=$1
        GROUP BY ta.id,u.full_name
        "#,
    )
    .bind(tahun_id)
    .fetch_optional(pool)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?)
}

async fn recalculate_akm(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tahun_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        WITH target AS (
            SELECT id,tanggal_selesai FROM tahun_akademik WHERE id=$1
        ), semester AS (
            SELECT e.registrasi_id,
                   COALESCE(ROUND(SUM(e.nilai_indeks*mk.sks)/NULLIF(SUM(mk.sks),0),2),0) AS ips,
                   COALESCE(SUM(mk.sks),0)::INT AS sks_semester
            FROM enrollments e JOIN jadwal_kuliah jk ON jk.id=e.jadwal_kuliah_id
            JOIN mata_kuliah mk ON mk.id=jk.matakuliah_id
            WHERE e.tahun_akademik_id=$1 AND e.status_approval::TEXT='Disetujui'
              AND e.nilai_indeks IS NOT NULL
            GROUP BY e.registrasi_id
        ), latest AS (
            SELECT DISTINCT ON (e.registrasi_id,jk.matakuliah_id)
                   e.registrasi_id,jk.matakuliah_id,mk.sks,e.nilai_indeks
            FROM enrollments e JOIN jadwal_kuliah jk ON jk.id=e.jadwal_kuliah_id
            JOIN mata_kuliah mk ON mk.id=jk.matakuliah_id
            JOIN tahun_akademik ta ON ta.id=e.tahun_akademik_id
            CROSS JOIN target t
            WHERE ta.tanggal_selesai<=t.tanggal_selesai AND e.nilai_indeks IS NOT NULL
              AND e.status_approval::TEXT='Disetujui'
            ORDER BY e.registrasi_id,jk.matakuliah_id,ta.tanggal_selesai DESC
        ), cumulative AS (
            SELECT registrasi_id,
                   COALESCE(ROUND(SUM(nilai_indeks*sks)/NULLIF(SUM(sks),0),2),0) AS ipk,
                   COALESCE(SUM(sks),0)::INT AS sks_total
            FROM latest GROUP BY registrasi_id
        )
        INSERT INTO aktivitas_kuliah_mahasiswa (
            registrasi_id,tahun_akademik_id,status_mahasiswa,ips,ipk,sks_semester,sks_total
        )
        SELECT s.registrasi_id,$1,rm.status_mahasiswa,s.ips,c.ipk,s.sks_semester,c.sks_total
        FROM semester s JOIN cumulative c ON c.registrasi_id=s.registrasi_id
        JOIN registrasi_mahasiswa rm ON rm.id=s.registrasi_id
        ON CONFLICT (registrasi_id,tahun_akademik_id) DO UPDATE SET
            status_mahasiswa=EXCLUDED.status_mahasiswa,ips=EXCLUDED.ips,ipk=EXCLUDED.ipk,
            sks_semester=EXCLUDED.sks_semester,sks_total=EXCLUDED.sks_total,updated_at=now()
        "#,
    )
    .bind(tahun_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn close(pool: &DbPool, tahun_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    let preview = status(pool, tahun_id).await?;
    if preview.status_penutupan == "Ditutup" {
        return Err(AppError::BadRequest("Semester sudah ditutup.".to_string()));
    }
    if preview.kelas_belum_siap > 0 {
        return Err(AppError::BadRequest(format!(
            "Masih ada {} kelas yang nilainya belum dipublikasikan.",
            preview.kelas_belum_siap
        )));
    }
    let mut tx = pool.begin().await?;
    recalculate_akm(&mut tx, tahun_id).await?;
    sqlx::query(
        r#"
        INSERT INTO feeder_sync_outbox(entity_type,entity_id,payload)
        SELECT 'AKM',akm.id,jsonb_build_object(
            'id_registrasi',akm.registrasi_id,'id_semester',ta.id_semester_feeder,
            'status_mahasiswa',akm.status_mahasiswa,'ips',akm.ips,'ipk',akm.ipk,
            'sks_semester',akm.sks_semester,'sks_total',akm.sks_total)
        FROM aktivitas_kuliah_mahasiswa akm JOIN tahun_akademik ta ON ta.id=akm.tahun_akademik_id
        WHERE akm.tahun_akademik_id=$1
        ON CONFLICT(entity_type,entity_id,operation) DO UPDATE SET
            payload=EXCLUDED.payload,status='Menunggu',last_error=NULL,updated_at=now()
        "#,
    )
    .bind(tahun_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO feeder_sync_outbox(entity_type,entity_id,payload)
        SELECT 'NILAI',e.id,jsonb_build_object(
            'id_peserta_kelas',e.id_peserta_kelas_feeder,'nilai_angka',e.nilai_angka,
            'nilai_huruf',e.nilai_huruf,'nilai_indeks',e.nilai_indeks)
        FROM enrollments e WHERE e.tahun_akademik_id=$1 AND e.nilai_angka IS NOT NULL
        ON CONFLICT(entity_type,entity_id,operation) DO UPDATE SET
            payload=EXCLUDED.payload,status='Menunggu',last_error=NULL,updated_at=now()
        "#,
    )
    .bind(tahun_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE tahun_akademik SET status_penutupan='Ditutup',ditutup_oleh=$1,ditutup_pada=now(),is_active=false,updated_at=now() WHERE id=$2")
        .bind(user_id).bind(tahun_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn khs(pool: &DbPool, user_id: Uuid, tahun_id: Uuid) -> Result<KhsResponse, AppError> {
    let identity = sqlx::query_as::<_, (String, String, String, String)>(
        r#"SELECT rm.nim,m.nama_mahasiswa,p.nama_prodi,ta.nama
        FROM mahasiswa m JOIN registrasi_mahasiswa rm ON rm.mahasiswa_id=m.id
        JOIN prodi p ON p.id=rm.prodi_id CROSS JOIN tahun_akademik ta
        WHERE m.user_id=$1 AND ta.id=$2"#,
    )
    .bind(user_id)
    .bind(tahun_id)
    .fetch_optional(pool)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;
    let status_penutupan =
        sqlx::query_scalar::<_, String>("SELECT status_penutupan FROM tahun_akademik WHERE id=$1")
            .bind(tahun_id)
            .fetch_one(pool)
            .await?;
    let summary = sqlx::query_as::<_, RingkasanAkademik>(
        r#"
        SELECT akm.ips,akm.ipk,akm.sks_semester,akm.sks_total,akm.status_mahasiswa
        FROM aktivitas_kuliah_mahasiswa akm JOIN registrasi_mahasiswa rm ON rm.id=akm.registrasi_id
        JOIN mahasiswa m ON m.id=rm.mahasiswa_id WHERE m.user_id=$1 AND akm.tahun_akademik_id=$2"#,
    )
    .bind(user_id)
    .bind(tahun_id)
    .fetch_optional(pool)
    .await?;
    let courses = sqlx::query_as::<_,KhsMataKuliah>(r#"
        SELECT mk.kode_mk,mk.nama_mk,mk.sks,e.nilai_angka,e.nilai_huruf,e.nilai_indeks,
               ROUND(e.nilai_indeks*mk.sks,2) AS mutu
        FROM enrollments e JOIN registrasi_mahasiswa rm ON rm.id=e.registrasi_id
        JOIN mahasiswa m ON m.id=rm.mahasiswa_id JOIN jadwal_kuliah jk ON jk.id=e.jadwal_kuliah_id
        JOIN mata_kuliah mk ON mk.id=jk.matakuliah_id JOIN nilai_akhir_kuliah na ON na.jadwal_kuliah_id=jk.id
        WHERE m.user_id=$1 AND e.tahun_akademik_id=$2 AND na.status::TEXT='Dipublikasikan'
          AND e.nilai_angka IS NOT NULL ORDER BY mk.kode_mk"#)
        .bind(user_id).bind(tahun_id).fetch_all(pool).await?;
    Ok(KhsResponse {
        tahun_akademik: identity.3,
        nim: identity.0,
        nama_mahasiswa: identity.1,
        nama_prodi: identity.2,
        status_penutupan,
        ringkasan: summary,
        mata_kuliah: courses,
    })
}

pub async fn transcript(pool: &DbPool, user_id: Uuid) -> Result<TranskripResponse, AppError> {
    let identity = sqlx::query_as::<_,(String,String,String)>(r#"SELECT rm.nim,m.nama_mahasiswa,p.nama_prodi
        FROM mahasiswa m JOIN registrasi_mahasiswa rm ON rm.mahasiswa_id=m.id JOIN prodi p ON p.id=rm.prodi_id
        WHERE m.user_id=$1"#).bind(user_id).fetch_optional(pool).await?.ok_or(sqlx::Error::RowNotFound)?;
    let courses = sqlx::query_as::<_,KhsMataKuliah>(r#"
        SELECT kode_mk,nama_mk,sks,nilai_angka,nilai_huruf,nilai_indeks,
               ROUND(nilai_indeks*sks,2) AS mutu FROM (
            SELECT DISTINCT ON (jk.matakuliah_id) mk.kode_mk,mk.nama_mk,mk.sks,
                   e.nilai_angka,e.nilai_huruf,e.nilai_indeks,ta.tanggal_selesai
            FROM enrollments e JOIN registrasi_mahasiswa rm ON rm.id=e.registrasi_id
            JOIN mahasiswa m ON m.id=rm.mahasiswa_id JOIN jadwal_kuliah jk ON jk.id=e.jadwal_kuliah_id
            JOIN mata_kuliah mk ON mk.id=jk.matakuliah_id JOIN tahun_akademik ta ON ta.id=e.tahun_akademik_id
            JOIN nilai_akhir_kuliah na ON na.jadwal_kuliah_id=jk.id
            WHERE m.user_id=$1 AND e.nilai_indeks IS NOT NULL AND na.status::TEXT='Dipublikasikan'
            ORDER BY jk.matakuliah_id,ta.tanggal_selesai DESC
        ) latest ORDER BY kode_mk"#).bind(user_id).fetch_all(pool).await?;
    let total_sks: i32 = courses.iter().map(|c| c.sks).sum();
    let total_mutu: Decimal = courses.iter().map(|c| c.mutu).sum();
    let ipk = if total_sks > 0 {
        (total_mutu / Decimal::from(total_sks)).round_dp(2)
    } else {
        Decimal::ZERO
    };
    Ok(TranskripResponse {
        nim: identity.0,
        nama_mahasiswa: identity.1,
        nama_prodi: identity.2,
        ipk,
        total_sks,
        mata_kuliah: courses,
    })
}

pub async fn outbox(pool: &DbPool) -> Result<Vec<FeederOutboxRow>, AppError> {
    Ok(sqlx::query_as::<_, FeederOutboxRow>(
        r#"SELECT id,entity_type,entity_id,operation,payload,
        status::TEXT AS status,attempts,last_error,created_at,synced_at FROM feeder_sync_outbox
        ORDER BY CASE status WHEN 'Gagal' THEN 0 WHEN 'Menunggu' THEN 1 ELSE 2 END,created_at"#,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn feeder_result(
    pool: &DbPool,
    id: Uuid,
    payload: FeederResultPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let entity = sqlx::query_as::<_, (String, Uuid)>(
        "SELECT entity_type,entity_id FROM feeder_sync_outbox WHERE id=$1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;
    let affected=sqlx::query(r#"UPDATE feeder_sync_outbox SET status=CASE WHEN $1 THEN 'Berhasil'::"StatusSinkronisasiFeeder" ELSE 'Gagal'::"StatusSinkronisasiFeeder" END,
        attempts=attempts+1,last_error=$2,synced_at=CASE WHEN $1 THEN now() ELSE NULL END,
        next_attempt_at=CASE WHEN $1 THEN NULL ELSE now()+interval '15 minutes' END,updated_at=now() WHERE id=$3"#)
        .bind(payload.berhasil).bind(payload.error).bind(id).execute(&mut *tx).await?.rows_affected();
    if payload.berhasil {
        if let Some(feeder_id) = payload.feeder_id {
            match entity.0.as_str() {
                "AKM" => {
                    sqlx::query("UPDATE aktivitas_kuliah_mahasiswa SET id_aktivitas_kuliah_feeder=$1,updated_at=now() WHERE id=$2").bind(feeder_id).bind(entity.1).execute(&mut *tx).await?;
                }
                "NILAI" => {
                    sqlx::query(
                        "UPDATE enrollments SET id_nilai_feeder=$1,updated_at=now() WHERE id=$2",
                    )
                    .bind(feeder_id)
                    .bind(entity.1)
                    .execute(&mut *tx)
                    .await?;
                }
                _ => {}
            }
        }
    }
    if affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    tx.commit().await?;
    Ok(())
}

pub async fn corrections(
    pool: &DbPool,
    claims: &TokenClaims,
) -> Result<Vec<KoreksiNilaiRow>, AppError> {
    let super_admin = access::has_role(claims, "SUPER_ADMIN");
    let academic = super_admin || access::has_role(claims, "STAF_AKADEMIK");
    let is_kaprodi = super_admin || access::has_role(claims, "KAPRODI");
    let prodi_ids = if is_kaprodi && !super_admin {
        access::kaprodi_prodi_ids(pool, claims.sub).await?
    } else {
        Vec::new()
    };
    let dosen_id = access::dosen_id(pool, claims.sub).await?;
    Ok(sqlx::query_as::<_,KoreksiNilaiRow>(r#"
        SELECT k.id,k.enrollment_id,rm.nim,m.nama_mahasiswa,mk.kode_mk,mk.nama_mk,
               k.nilai_angka_lama,k.nilai_huruf_lama,k.nilai_angka_baru,k.nilai_huruf_baru,
               k.alasan,k.status::TEXT AS status,u.full_name AS diajukan_oleh,
               k.catatan_review,k.created_at
        FROM koreksi_nilai k JOIN enrollments e ON e.id=k.enrollment_id
        JOIN registrasi_mahasiswa rm ON rm.id=e.registrasi_id JOIN mahasiswa m ON m.id=rm.mahasiswa_id
        JOIN jadwal_kuliah jk ON jk.id=e.jadwal_kuliah_id JOIN mata_kuliah mk ON mk.id=jk.matakuliah_id
        JOIN users u ON u.id=k.diajukan_oleh
        WHERE $1 OR ($2 AND mk.prodi_id=ANY($3)) OR EXISTS(
            SELECT 1 FROM jadwal_dosen_pengampu jdp
            WHERE jdp.jadwal_kuliah_id=jk.id AND jdp.dosen_id=$4
        ) ORDER BY k.created_at DESC"#)
        .bind(academic).bind(is_kaprodi).bind(&prodi_ids).bind(dosen_id).fetch_all(pool).await?)
}

pub async fn submit_correction(
    pool: &DbPool,
    claims: &TokenClaims,
    payload: AjukanKoreksiNilaiPayload,
) -> Result<(), AppError> {
    if payload.alasan.trim().len() < 10
        || payload.nilai_angka_baru < Decimal::ZERO
        || payload.nilai_angka_baru > Decimal::from(100)
    {
        return Err(AppError::BadRequest(
            "Alasan minimal 10 karakter dan nilai harus 0–100.".to_string(),
        ));
    }
    let jadwal_id =
        sqlx::query_scalar::<_, Uuid>("SELECT jadwal_kuliah_id FROM enrollments WHERE id=$1")
            .bind(payload.enrollment_id)
            .fetch_optional(pool)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;
    let permission = access::for_jadwal(pool, claims, jadwal_id).await?;
    access::require_grade(&permission)?;
    let current = sqlx::query_as::<
        _,
        (
            Option<Decimal>,
            Option<String>,
            Option<Decimal>,
            String,
            String,
        ),
    >(
        r#"
        SELECT e.nilai_angka,e.nilai_huruf,e.nilai_indeks,na.status::TEXT,ta.status_penutupan
        FROM enrollments e JOIN jadwal_kuliah jk ON jk.id=e.jadwal_kuliah_id
        JOIN tahun_akademik ta ON ta.id=e.tahun_akademik_id
        JOIN nilai_akhir_kuliah na ON na.jadwal_kuliah_id=jk.id WHERE e.id=$1"#,
    )
    .bind(payload.enrollment_id)
    .fetch_one(pool)
    .await?;
    if current.3 != "Dipublikasikan" {
        return Err(AppError::BadRequest(
            "Nilai belum dipublikasikan.".to_string(),
        ));
    }
    let scale=sqlx::query_as::<_,(String,Decimal)>(r#"
        SELECT sn.nilai_huruf,sn.nilai_indeks FROM skala_nilai sn
        JOIN mata_kuliah mk ON mk.prodi_id=sn.prodi_id JOIN jadwal_kuliah jk ON jk.matakuliah_id=mk.id
        JOIN enrollments e ON e.jadwal_kuliah_id=jk.id JOIN tahun_akademik ta ON ta.id=e.tahun_akademik_id
        WHERE e.id=$1 AND $2 BETWEEN sn.bobot_minimum AND sn.bobot_maksimum
          AND sn.tanggal_mulai_efektif<=ta.tanggal_selesai
          AND (sn.tanggal_akhir_efektif IS NULL OR sn.tanggal_akhir_efektif>=ta.tanggal_mulai)
        ORDER BY sn.tanggal_mulai_efektif DESC LIMIT 1"#)
        .bind(payload.enrollment_id).bind(payload.nilai_angka_baru).fetch_optional(pool).await?
        .ok_or_else(||AppError::BadRequest("Nilai baru tidak tercakup skala Prodi.".to_string()))?;
    sqlx::query(
        r#"INSERT INTO koreksi_nilai(enrollment_id,nilai_angka_lama,nilai_huruf_lama,
        nilai_indeks_lama,nilai_angka_baru,nilai_huruf_baru,nilai_indeks_baru,alasan,diajukan_oleh)
        VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)"#,
    )
    .bind(payload.enrollment_id)
    .bind(current.0)
    .bind(current.1)
    .bind(current.2)
    .bind(payload.nilai_angka_baru)
    .bind(scale.0)
    .bind(scale.1)
    .bind(payload.alasan.trim())
    .bind(claims.sub)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn review_correction(
    pool: &DbPool,
    claims: &TokenClaims,
    id: Uuid,
    payload: ReviewKoreksiPayload,
) -> Result<(), AppError> {
    if !["Disetujui", "Ditolak"].contains(&payload.aksi.as_str()) {
        return Err(AppError::BadRequest("Aksi review tidak valid.".to_string()));
    }
    let jadwal_id = sqlx::query_scalar::<_, Uuid>(
        r#"SELECT e.jadwal_kuliah_id FROM koreksi_nilai k
        JOIN enrollments e ON e.id=k.enrollment_id WHERE k.id=$1"#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    let permission = access::for_jadwal(pool, claims, jadwal_id).await?;
    if !permission.kaprodi {
        return Err(AppError::Forbidden(
            "Hanya Kaprodi terkait yang dapat meninjau koreksi.".to_string(),
        ));
    }
    let affected = sqlx::query(
        r#"UPDATE koreksi_nilai SET status=$1::"StatusKoreksiNilai",
        catatan_review=$2,ditinjau_oleh=$3,ditinjau_pada=now(),updated_at=now()
        WHERE id=$4 AND status='Diajukan'"#,
    )
    .bind(payload.aksi)
    .bind(payload.catatan)
    .bind(claims.sub)
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(AppError::BadRequest(
            "Koreksi tidak sedang diajukan.".to_string(),
        ));
    }
    Ok(())
}

pub async fn apply_correction(
    pool: &DbPool,
    claims: &TokenClaims,
    id: Uuid,
) -> Result<(), AppError> {
    if !(access::has_role(claims, "SUPER_ADMIN") || access::has_role(claims, "STAF_AKADEMIK")) {
        return Err(AppError::Forbidden(
            "Hanya Staf Akademik yang dapat menerapkan koreksi.".to_string(),
        ));
    }
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as::<_, (Uuid, Decimal, String, Decimal, Uuid, Uuid)>(
        r#"SELECT k.enrollment_id,
        k.nilai_angka_baru,k.nilai_huruf_baru,k.nilai_indeks_baru,
        e.tahun_akademik_id,e.registrasi_id
        FROM koreksi_nilai k JOIN enrollments e ON e.id=k.enrollment_id
        WHERE k.id=$1 AND k.status='Disetujui' FOR UPDATE"#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        AppError::BadRequest("Koreksi belum disetujui atau sudah diterapkan.".to_string())
    })?;
    sqlx::query("UPDATE enrollments SET nilai_angka=$1,nilai_huruf=$2,nilai_indeks=$3,updated_at=now() WHERE id=$4")
        .bind(row.1).bind(&row.2).bind(row.3).bind(row.0).execute(&mut *tx).await?;
    recalculate_akm(&mut tx, row.4).await?;
    let later_periods=sqlx::query_scalar::<_,Uuid>(r#"SELECT akm.tahun_akademik_id
        FROM aktivitas_kuliah_mahasiswa akm JOIN tahun_akademik ta ON ta.id=akm.tahun_akademik_id
        WHERE akm.registrasi_id=$1 AND ta.tanggal_selesai>(SELECT tanggal_selesai FROM tahun_akademik WHERE id=$2)
        ORDER BY ta.tanggal_selesai"#).bind(row.5).bind(row.4).fetch_all(&mut *tx).await?;
    for period_id in later_periods {
        recalculate_akm(&mut tx, period_id).await?;
    }
    sqlx::query(
        r#"INSERT INTO feeder_sync_outbox(entity_type,entity_id,payload)
        SELECT 'NILAI',e.id,jsonb_build_object('id_peserta_kelas',e.id_peserta_kelas_feeder,
        'nilai_angka',e.nilai_angka,'nilai_huruf',e.nilai_huruf,'nilai_indeks',e.nilai_indeks)
        FROM enrollments e WHERE e.id=$1 ON CONFLICT(entity_type,entity_id,operation) DO UPDATE SET
        payload=EXCLUDED.payload,status='Menunggu',last_error=NULL,updated_at=now()"#,
    )
    .bind(row.0)
    .execute(&mut *tx)
    .await?;
    sqlx::query(r#"INSERT INTO feeder_sync_outbox(entity_type,entity_id,payload)
        SELECT 'AKM',akm.id,jsonb_build_object('id_registrasi',akm.registrasi_id,
        'id_semester',ta.id_semester_feeder,'status_mahasiswa',akm.status_mahasiswa,
        'ips',akm.ips,'ipk',akm.ipk,'sks_semester',akm.sks_semester,'sks_total',akm.sks_total)
        FROM aktivitas_kuliah_mahasiswa akm JOIN tahun_akademik ta ON ta.id=akm.tahun_akademik_id
        WHERE akm.registrasi_id=$1 AND ta.tanggal_selesai>=(SELECT tanggal_selesai FROM tahun_akademik WHERE id=$2)
        ON CONFLICT(entity_type,entity_id,operation) DO UPDATE SET payload=EXCLUDED.payload,
        status='Menunggu',last_error=NULL,updated_at=now()"#)
        .bind(row.5).bind(row.4).execute(&mut *tx).await?;
    sqlx::query("UPDATE koreksi_nilai SET status='Diterapkan',diterapkan_oleh=$1,diterapkan_pada=now(),updated_at=now() WHERE id=$2")
        .bind(claims.sub).bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}
