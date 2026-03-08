// src/modules/krs/repo.rs
use crate::{db::DbPool, errors::AppError, modules::tahun_akademik::model::TahunAkademik};

use super::model::{
    CreateEnrollmentPayload, EnrollmentDetail, EnrollmentFromDb, EnrollmentStatus,
    UpdateEnrollmentStatusPayload, UpdateNilaiPayload,
};
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn create_enrollment_repo(
    pool: &DbPool,
    registrasi_id: Uuid,
    payload: CreateEnrollmentPayload,
) -> Result<(), AppError> {
    let today = OffsetDateTime::now_utc().date();
    let ta = sqlx::query_as!(
        TahunAkademik,
        "SELECT * FROM tahun_akademik WHERE id = $1",
        payload.tahun_akademik_id
    )
    .fetch_one(pool)
    .await?;

    if !(today >= ta.krs_mulai && today <= ta.krs_selesai) {
        let error_message = format!(
            "Periode pengisian KRS untuk {} sudah ditutup (berlaku dari {} hingga {}).",
            ta.nama, ta.krs_mulai, ta.krs_selesai
        );
        return Err(AppError::Forbidden(error_message));
    }

    sqlx::query!(
        "INSERT INTO enrollments (registrasi_id, matakuliah_id, tahun_akademik_id) VALUES ($1, $2, $3)",
        registrasi_id, payload.matakuliah_id, payload.tahun_akademik_id
    )
    .execute(pool)
    .await?;

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
    // Menggunakan query_as! dengan EnrollmentFromDb untuk mencegah error 'type annotation needed'
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
        LEFT JOIN mata_kuliah mk ON e.matakuliah_id = mk.id
        LEFT JOIN tahun_akademik ta ON e.tahun_akademik_id = ta.id
        WHERE e.registrasi_id = $1 AND e.tahun_akademik_id = $2
        ORDER BY mk.kode_mk
        "#,
        registrasi_id,
        tahun_akademik_id
    )
    .fetch_all(pool)
    .await?;

    // Konversi otomatis karena kita sudah implementasi `From<EnrollmentFromDb>` di model
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
        LEFT JOIN mata_kuliah mk ON e.matakuliah_id = mk.id
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
