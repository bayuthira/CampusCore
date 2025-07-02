// src/repositories/krs_repo.rs

use crate::{
    db::DbPool,
    errors::AppError,
    models::{krs_model::{
        CreateEnrollmentPayload, EnrollmentDetail, EnrollmentFromDb, UpdateEnrollmentStatusPayload,EnrollmentStatus
    },
        tahun_akademik_model::TahunAkademik,
    },
};
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn create_enrollment_repo(
    pool: &DbPool,
    mahasiswa_id: Uuid,
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
        "INSERT INTO enrollments (mahasiswa_id, matakuliah_id, tahun_akademik_id) VALUES ($1, $2, $3)",
        mahasiswa_id, payload.matakuliah_id, payload.tahun_akademik_id
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
    mahasiswa_id: Uuid,
    tahun_akademik_id: Uuid,
) -> Result<Vec<EnrollmentDetail>, AppError> {
    // Gunakan query_as! untuk memetakan hasil langsung ke struct EnrollmentFromDb.
    // Ini lebih aman dan memanfaatkan struct yang sudah Anda definisikan.
    let enrollments_from_db = sqlx::query_as!(
        EnrollmentFromDb,
        r#"
        SELECT 
            e.id, 
            e.mahasiswa_id,
            ta.nama as tahun_akademik,
            mk.kode_mk, 
            mk.nama_mk, 
            mk.sks, 
            e.status_approval::TEXT as "status_approval!", -- Pastikan tipe enum di DB menjadi String di Rust
            e.nilai_huruf
        FROM enrollments e
        LEFT JOIN mata_kuliah mk ON e.matakuliah_id = mk.id
        LEFT JOIN tahun_akademik ta ON e.tahun_akademik_id = ta.id
        WHERE e.mahasiswa_id = $1 AND e.tahun_akademik_id = $2
        ORDER BY mk.kode_mk
        "#, mahasiswa_id, tahun_akademik_id
    )
    .fetch_all(pool)
    .await?;

    // Konversi Vec<EnrollmentFromDb> ke Vec<EnrollmentDetail> menggunakan implementasi `From`
    let enrollments_detail: Vec<EnrollmentDetail> = enrollments_from_db
        .into_iter()
        .map(Into::into) // Ini akan memanggil `EnrollmentDetail::from(enrollment_db_item)`
        .collect();

    Ok(enrollments_detail)
}

pub async fn get_enrollment_by_id_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<EnrollmentDetail, AppError> {
    // Sama seperti di atas, gunakan query_as! untuk keamanan tipe.
    let enrollment_from_db = sqlx::query_as!(
        EnrollmentFromDb,
        r#"
        SELECT 
            e.id, 
            e.mahasiswa_id,
            ta.nama as tahun_akademik,
            mk.kode_mk, 
            mk.nama_mk, 
            mk.sks, 
            e.status_approval::TEXT as "status_approval!",
            e.nilai_huruf
        FROM enrollments e
        LEFT JOIN mata_kuliah mk ON e.matakuliah_id = mk.id
        LEFT JOIN tahun_akademik ta ON e.tahun_akademik_id = ta.id
        WHERE e.id = $1
        "#, id
    )
    .fetch_one(pool)
    .await?;

    // Konversi satu objek dari EnrollmentFromDb ke EnrollmentDetail
    let enrollment_detail: EnrollmentDetail = enrollment_from_db.into();

    Ok(enrollment_detail)
}

pub async fn update_enrollment_status_repo(
    pool: &DbPool,
    enrollment_id: Uuid,
    payload: UpdateEnrollmentStatusPayload,
) -> Result<EnrollmentDetail, AppError> {
    // Gunakan `sqlx::query!` karena kita perlu passing enum secara eksplisit
    sqlx::query!(
        "UPDATE enrollments SET status_approval = $1, updated_at = now() WHERE id = $2",
        payload.status_approval as EnrollmentStatus, // Casting ke tipe ENUM
        enrollment_id
    )
    .execute(pool)
    .await?;

    // Ambil dan kembalikan data terbaru setelah diupdate
    let updated_enrollment = get_enrollment_by_id_repo(pool, enrollment_id).await?;
    Ok(updated_enrollment)
}