// src/repositories/krs_repo.rs

use crate::{db::DbPool, errors::AppError, modules::tahun_akademik::model::TahunAkademik};

use super::model::{
    CreateEnrollmentPayload, EnrollmentDetail, EnrollmentStatus, UpdateEnrollmentStatusPayload,
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
    // Query ini TIDAK BERUBAH. Ini sudah benar.
    let rows = sqlx::query!(
        r#"
        SELECT 
            e.id, 
            e.mahasiswa_id,
            COALESCE(ta.nama, '') as "tahun_akademik!",
            COALESCE(mk.kode_mk, '') as "kode_mk!",
            COALESCE(mk.nama_mk, '') as "nama_mk!",
            COALESCE(mk.sks, 0) as "sks!",
            e.status_approval::TEXT as "status_approval", 
            e.nilai_huruf
        FROM enrollments e
        LEFT JOIN mata_kuliah mk ON e.matakuliah_id = mk.id
        LEFT JOIN tahun_akademik ta ON e.tahun_akademik_id = ta.id
        WHERE e.mahasiswa_id = $1 AND e.tahun_akademik_id = $2
        ORDER BY mk.kode_mk
        "#,
        mahasiswa_id,
        tahun_akademik_id
    )
    .fetch_all(pool)
    .await?;

    // --- PERBAIKAN DI BLOK .map() DI BAWAH INI ---
    let enrollments_detail: Vec<EnrollmentDetail> = rows
        .into_iter()
        .map(|row| {
            let status = match row.status_approval.as_deref() {
                // status_approval masih Option karena bisa NULL
                Some("Disetujui") => EnrollmentStatus::Disetujui,
                Some("Ditolak") => EnrollmentStatus::Ditolak,
                Some("Selesai") => EnrollmentStatus::Selesai,
                Some("Mengulang") => EnrollmentStatus::Mengulang,
                _ => EnrollmentStatus::MenungguPersetujuan,
            };

            EnrollmentDetail {
                id: row.id,
                mahasiswa_id: row.mahasiswa_id,
                // Hapus .unwrap_or_default() karena `row` sudah berisi tipe non-option
                tahun_akademik: row.tahun_akademik,
                kode_mk: row.kode_mk,
                nama_mk: row.nama_mk,
                sks: row.sks,
                status_approval: status,
                nilai_huruf: row.nilai_huruf,
            }
        })
        .collect();

    Ok(enrollments_detail)
}

pub async fn get_enrollment_by_id_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<EnrollmentDetail, AppError> {
    // Menggunakan query! dengan COALESCE dan '!' persis seperti fungsi Anda yang lain
    let row = sqlx::query!(
        r#"
        SELECT 
            e.id, 
            e.mahasiswa_id,
            COALESCE(ta.nama, '') as "tahun_akademik!",
            COALESCE(mk.kode_mk, '') as "kode_mk!",
            COALESCE(mk.nama_mk, '') as "nama_mk!",
            COALESCE(mk.sks, 0) as "sks!",
            e.status_approval::TEXT as "status_approval", 
            e.nilai_huruf
        FROM enrollments e
        LEFT JOIN mata_kuliah mk ON e.matakuliah_id = mk.id
        LEFT JOIN tahun_akademik ta ON e.tahun_akademik_id = ta.id
        WHERE e.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;

    // Melakukan pemetaan manual, persis seperti yang Anda lakukan
    let status = match row.status_approval.as_deref() {
        Some("Disetujui") => EnrollmentStatus::Disetujui,
        Some("Ditolak") => EnrollmentStatus::Ditolak,
        Some("Selesai") => EnrollmentStatus::Selesai,
        Some("Mengulang") => EnrollmentStatus::Mengulang,
        _ => EnrollmentStatus::MenungguPersetujuan,
    };

    // Membuat struct EnrollmentDetail TANPA .unwrap_or_default()
    let enrollment_detail = EnrollmentDetail {
        id: row.id,
        mahasiswa_id: row.mahasiswa_id,
        tahun_akademik: row.tahun_akademik, // Langsung digunakan
        kode_mk: row.kode_mk,               // Langsung digunakan
        nama_mk: row.nama_mk,               // Langsung digunakan
        sks: row.sks,                       // Langsung digunakan
        status_approval: status,
        nilai_huruf: row.nilai_huruf,
    };

    Ok(enrollment_detail)
}

pub async fn update_enrollment_status_repo(
    pool: &DbPool,
    enrollment_id: Uuid,
    payload: UpdateEnrollmentStatusPayload,
) -> Result<(), AppError> {
    // <-- Return type diubah menjadi Result<(), AppError>
    // 1. Konversi enum dari payload ke representasi string yang TEPAT
    //    sesuai dengan label yang ada di ENUM PostgreSQL Anda.
    //    Perhatikan "Menunggu Persetujuan" yang kemungkinan memiliki spasi.
    let status_str = match payload.status_approval {
        EnrollmentStatus::MenungguPersetujuan => "Menunggu Persetujuan",
        EnrollmentStatus::Disetujui => "Disetujui",
        EnrollmentStatus::Ditolak => "Ditolak",
        EnrollmentStatus::Selesai => "Selesai",
        EnrollmentStatus::Mengulang => "Mengulang",
    };

    // 2. Jalankan query UPDATE dengan melakukan CAST eksplisit dari string ke tipe ENUM di database.
    //    Ini adalah cara yang paling robust untuk memastikan PostgreSQL menerima nilainya.
    //    Kita menggunakan sqlx::query() (non-macro) untuk menghindari type-checking dari macro yang terlalu ketat.
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

    // Jika tidak ada baris yang ter-update, berarti id tidak ditemukan
    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    // Jika berhasil, cukup kembalikan Ok
    Ok(())
}
