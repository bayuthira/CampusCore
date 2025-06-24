// src/repositories/krs_repo.rs

use crate::{
    db::DbPool,
    errors::AppError,
    models::{
        krs_model::{CreateEnrollmentPayload, EnrollmentDetail},
        tahun_akademik_model::TahunAkademik,
    },
};
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn create_enrollment_repo(
    pool: &DbPool,
    mahasiswa_id: Uuid,
    payload: CreateEnrollmentPayload,
) -> Result<EnrollmentDetail, AppError> {
    // Validasi Logika Bisnis: Apakah periode KRS masih buka?
    let today = OffsetDateTime::now_utc().date();
    let ta = sqlx::query_as!(
        TahunAkademik,
        "SELECT * FROM tahun_akademik WHERE id = $1",
        payload.tahun_akademik_id
    )
    .fetch_one(pool)
    .await?;

    if !(today >= ta.krs_mulai && today <= ta.krs_selesai) {
        return Err(AppError::Forbidden); // Atau error lain yang lebih spesifik
    }

    let enrollment_id = sqlx::query_scalar!(
        "INSERT INTO enrollments (mahasiswa_id, matakuliah_id, tahun_akademik_id) VALUES ($1, $2, $3) RETURNING id",
        mahasiswa_id, payload.matakuliah_id, payload.tahun_akademik_id
    )
    .fetch_one(pool)
    .await?;

    let detail = get_enrollment_by_id_repo(pool, enrollment_id).await?;
    Ok(detail)
}

pub async fn get_my_enrollments_repo(
    pool: &DbPool,
    mahasiswa_id: Uuid,
    tahun_akademik_id: Uuid,
) -> Result<Vec<EnrollmentDetail>, AppError> {
    let enrollments = sqlx::query_as!(
        EnrollmentDetail,
        r#"
        SELECT 
            e.id,
            ta.nama as "tahun_akademik",
            mk.kode_mk,
            mk.nama_mk,
            mk.sks,
            e.status_approval AS "status_approval: _",
            e.nilai_huruf
        FROM enrollments e
        JOIN mata_kuliah mk ON e.matakuliah_id = mk.id
        JOIN tahun_akademik ta ON e.tahun_akademik_id = ta.id
        WHERE e.mahasiswa_id = $1 AND e.tahun_akademik_id = $2
        ORDER BY mk.kode_mk
        "#,
        mahasiswa_id,
        tahun_akademik_id
    )
    .fetch_all(pool)
    .await?;
    Ok(enrollments)
}

pub async fn get_enrollment_by_id_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<EnrollmentDetail, AppError> {
    let enrollment = sqlx::query_as!(
        EnrollmentDetail,
        r#"
        SELECT 
            e.id,
            ta.nama as "tahun_akademik",
            mk.kode_mk,
            mk.nama_mk,
            mk.sks,
            e.status_approval AS "status_approval: _",
            e.nilai_huruf
        FROM enrollments e
        JOIN mata_kuliah mk ON e.matakuliah_id = mk.id
        JOIN tahun_akademik ta ON e.tahun_akademik_id = ta.id
        WHERE e.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(enrollment)
}