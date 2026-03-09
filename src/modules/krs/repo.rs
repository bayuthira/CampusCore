// src/modules/krs/repo.rs
use crate::{db::DbPool, errors::AppError, modules::tahun_akademik::model::TahunAkademik};

use super::model::{
    AvailableJadwalDetail, CreateEnrollmentPayload, EnrollmentDetail, EnrollmentFromDb,
    EnrollmentStatus, UpdateEnrollmentStatusPayload, UpdateNilaiPayload,
};
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn create_enrollment_repo(
    pool: &DbPool,
    registrasi_id: Uuid,
    payload: CreateEnrollmentPayload,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    let today = OffsetDateTime::now_utc().date();
    let ta = sqlx::query_as!(
        TahunAkademik,
        "SELECT * FROM tahun_akademik WHERE id = $1",
        payload.tahun_akademik_id
    )
    .fetch_one(&mut *tx)
    .await?;

    // Validasi Tanggal KRS
    if !(today >= ta.krs_mulai && today <= ta.krs_selesai) {
        return Err(AppError::Forbidden(
            "Periode pengisian KRS sudah ditutup.".to_string(),
        ));
    }

    // Loop array dari Frontend dan Insert
    // (Jika mau lebih advanced bisa pakai sqlx query builder untuk bulk insert,
    // tapi loop di dalam transaksi sudah cukup cepat untuk puluhan baris)
    for jadwal_id in payload.jadwal_kuliah_ids {
        // Validasi opsional: Cek apakah mahasiswa sudah ambil jadwal ini
        let is_duplicate = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM enrollments WHERE registrasi_id = $1 AND jadwal_kuliah_id = $2)",
            registrasi_id, jadwal_id
        ).fetch_one(&mut *tx).await?.unwrap_or(false);

        if !is_duplicate {
            sqlx::query!(
                "INSERT INTO enrollments (registrasi_id, jadwal_kuliah_id, tahun_akademik_id) VALUES ($1, $2, $3)",
                registrasi_id, jadwal_id, payload.tahun_akademik_id
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_enrollment_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM enrollments WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}

pub async fn get_my_enrollments_repo(
    pool: &DbPool,
    registrasi_id: Uuid,
    tahun_akademik_id: Uuid,
) -> Result<Vec<EnrollmentDetail>, AppError> {
    let rows = sqlx::query_as!(
        EnrollmentFromDb,
        r#"
        SELECT 
            e.id, 
            e.registrasi_id,
            ta.nama as "tahun_akademik",
            mk.kode_mk as "kode_mk",
            mk.nama_mk as "nama_mk",
            mk.sks as "sks",
            e.status_approval::TEXT as "status_approval!", 
            e.nilai_huruf,
            e.id_peserta_kelas_feeder,
            e.id_nilai_feeder,
            e.nilai_angka,
            e.nilai_indeks
        FROM enrollments e
        JOIN jadwal_kuliah jk ON e.jadwal_kuliah_id = jk.id -- JOIN BARU
        JOIN mata_kuliah mk ON jk.matakuliah_id = mk.id     -- PERUBAHAN JOIN
        LEFT JOIN tahun_akademik ta ON e.tahun_akademik_id = ta.id
        WHERE e.registrasi_id = $1 AND e.tahun_akademik_id = $2
        ORDER BY mk.kode_mk
        "#,
        registrasi_id,
        tahun_akademik_id
    )
    .fetch_all(pool)
    .await?;

    let enrollments_detail: Vec<EnrollmentDetail> =
        rows.into_iter().map(|row| row.into()).collect();
    Ok(enrollments_detail)
}

pub async fn get_enrollment_by_id_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<EnrollmentDetail, AppError> {
    let row = sqlx::query_as!(
        EnrollmentFromDb,
        r#"
        SELECT 
            e.id, 
            e.registrasi_id,
            ta.nama as "tahun_akademik",
            mk.kode_mk as "kode_mk",
            mk.nama_mk as "nama_mk",
            mk.sks as "sks",
            e.status_approval::TEXT as "status_approval!", 
            e.nilai_huruf,
            e.id_peserta_kelas_feeder,
            e.id_nilai_feeder,
            e.nilai_angka,
            e.nilai_indeks
        FROM enrollments e
        JOIN jadwal_kuliah jk ON e.jadwal_kuliah_id = jk.id
        JOIN mata_kuliah mk ON jk.matakuliah_id = mk.id
        LEFT JOIN tahun_akademik ta ON e.tahun_akademik_id = ta.id
        WHERE e.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn update_enrollment_status_repo(
    pool: &DbPool,
    enrollment_id: Uuid,
    payload: UpdateEnrollmentStatusPayload,
) -> Result<(), AppError> {
    let status_str = match payload.status_approval {
        EnrollmentStatus::MenungguPersetujuan => "Menunggu Persetujuan",
        EnrollmentStatus::Disetujui => "Disetujui",
        EnrollmentStatus::Ditolak => "Ditolak",
        EnrollmentStatus::Selesai => "Selesai",
        EnrollmentStatus::Mengulang => "Mengulang",
    };

    let rows_affected = sqlx::query(
        r#"
        UPDATE enrollments SET status_approval = $1::"EnrollmentStatus", updated_at = now() WHERE id = $2
        "#,
    )
    .bind(status_str)
    .bind(enrollment_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}

pub async fn update_nilai_repo(
    pool: &DbPool,
    enrollment_id: Uuid,
    payload: UpdateNilaiPayload,
) -> Result<(), AppError> {
    let rows_affected = sqlx::query(
        r#"
        UPDATE enrollments 
        SET nilai_angka = $1, 
            nilai_indeks = $2, 
            nilai_huruf = $3, 
            id_nilai_feeder = $4, 
            updated_at = now() 
        WHERE id = $5
        "#,
    )
    .bind(payload.nilai_angka)
    .bind(payload.nilai_indeks)
    .bind(payload.nilai_huruf)
    .bind(payload.id_nilai_feeder)
    .bind(enrollment_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}

// --- FUNGSI MENGAMBIL JADWAL AVAILABLE ---
pub async fn get_available_jadwal_repo(
    pool: &DbPool,
    tahun_akademik_id: Uuid,
    prodi_id: Uuid,
    kode_rombel: String, // String rombel milik mahasiswa (misal "A")
) -> Result<Vec<AvailableJadwalDetail>, AppError> {
    // Query ini sangat optimal karena langsung mengagregasi nama dosen
    // dan menghitung is_paket langsung di level Database.
    let jadwals = sqlx::query_as!(
        AvailableJadwalDetail,
        r#"
        SELECT 
            jk.id as jadwal_id,
            mk.id as matakuliah_id,
            mk.kode_mk,
            mk.nama_mk,
            mk.sks,
            mk.semester_target,
            jk.kelas,
            jk.nama_kelas_kuliah,
            jk.hari::TEXT as "hari!",
            jk.jam_mulai::TEXT as "jam_mulai!",
            jk.jam_selesai::TEXT as "jam_selesai!",
            (COALESCE(jk.nama_kelas_kuliah, jk.kelas) = $3) as "is_paket!",
            (
                SELECT string_agg(p.nama_lengkap, ', ')
                FROM jadwal_dosen_pengampu jdp
                JOIN dosen d ON jdp.dosen_id = d.id
                JOIN pegawai p ON d.pegawai_id = p.id
                WHERE jdp.jadwal_kuliah_id = jk.id
            ) as dosen_pengampu
        FROM jadwal_kuliah jk
        JOIN mata_kuliah mk ON jk.matakuliah_id = mk.id
        WHERE jk.tahun_akademik_id = $1 AND mk.prodi_id = $2
        ORDER BY mk.semester_target ASC, mk.nama_mk ASC, jk.kelas ASC
        "#,
        tahun_akademik_id,
        prodi_id,
        kode_rombel
    )
    .fetch_all(pool)
    .await?;

    Ok(jadwals)
}
