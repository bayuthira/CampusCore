// src/modules/krs/dosen_pa_repo.rs
use crate::{
    db::DbPool,
    errors::AppError,
    modules::mahasiswa::model::{
        BatchAssignDosenPaPayload, MahasiswaBimbingan, SingleAssignDosenPaPayload,
    },
};
use uuid::Uuid;

pub async fn get_my_advisees_repo(
    pool: &DbPool,
    dosen_pa_id: Uuid,
) -> Result<Vec<MahasiswaBimbingan>, AppError> {
    // PERBAIKAN: prodi_id, nim, dan angkatan sekarang diambil dari tabel registrasi_mahasiswa (rm),
    // bukan dari tabel mahasiswa (m).
    let advisees = sqlx::query_as!(
        MahasiswaBimbingan,
        r#"
        SELECT DISTINCT ON (rm.nim)
            m.id,
            rm.nim,
            m.nama_mahasiswa,
            rm.angkatan,
            m.email,
            p.nama_prodi
        FROM mahasiswa m
        JOIN registrasi_mahasiswa rm ON rm.mahasiswa_id = m.id
        JOIN prodi p ON rm.prodi_id = p.id
        WHERE rm.dosen_pa_id = $1
        ORDER BY rm.nim ASC, rm.created_at DESC
        "#,
        dosen_pa_id
    )
    .fetch_all(pool)
    .await?;

    Ok(advisees)
}

// --- FUNGSI BARU: BATCH ASSIGN ---
pub async fn batch_assign_dosen_pa_repo(
    pool: &DbPool,
    payload: BatchAssignDosenPaPayload,
) -> Result<u64, AppError> {
    let rows_affected = sqlx::query!(
        r#"
        UPDATE registrasi_mahasiswa
        SET dosen_pa_id = $1, updated_at = now()
        WHERE prodi_id = $2 AND angkatan = $3 AND kode_rombel = $4
        "#,
        payload.dosen_pa_id,
        payload.prodi_id,
        payload.angkatan,
        payload.kode_rombel
    )
    .execute(pool)
    .await?
    .rows_affected();

    // Mengembalikan jumlah baris yang berhasil diupdate
    Ok(rows_affected)
}

// --- FUNGSI BARU: SINGLE ASSIGN ---
pub async fn single_assign_dosen_pa_repo(
    pool: &DbPool,
    payload: SingleAssignDosenPaPayload,
) -> Result<(), AppError> {
    let rows_affected = sqlx::query!(
        r#"
        UPDATE registrasi_mahasiswa
        SET dosen_pa_id = $1, updated_at = now()
        WHERE id = $2
        "#,
        payload.dosen_pa_id,
        payload.registrasi_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    Ok(())
}
