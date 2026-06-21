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
    tahun_akademik_id: Uuid,
) -> Result<Vec<MahasiswaBimbingan>, AppError> {
    let advisees = sqlx::query_as!(
        MahasiswaBimbingan,
        r#"
        WITH mahasiswa_pa AS (
            SELECT DISTINCT ON (rm.mahasiswa_id)
                m.id, rm.id AS registrasi_id, rm.nim, m.nama_mahasiswa,
                rm.angkatan, m.email, p.nama_prodi
            FROM registrasi_mahasiswa rm
            JOIN mahasiswa m ON m.id = rm.mahasiswa_id
            JOIN prodi p ON p.id = rm.prodi_id
            WHERE rm.dosen_pa_id = $1
            ORDER BY rm.mahasiswa_id, rm.created_at DESC
        )
        SELECT mp.id, mp.nim, mp.nama_mahasiswa, mp.angkatan, mp.email,
               mp.nama_prodi,
               CASE
                   WHEN COUNT(e.id) = 0 THEN 'Belum Isi'
                   WHEN COUNT(e.id) FILTER (
                       WHERE e.status_approval::TEXT = 'Menunggu Persetujuan'
                   ) > 0 THEN 'Menunggu Persetujuan'
                   WHEN COUNT(e.id) FILTER (
                       WHERE e.status_approval::TEXT = 'Ditolak'
                   ) > 0 THEN 'Perlu Perbaikan'
                   WHEN COUNT(e.id) FILTER (
                       WHERE e.status_approval::TEXT = 'Disetujui'
                   ) = COUNT(e.id) THEN 'Disetujui'
                   ELSE 'Diproses'
               END AS "status_krs!",
               COUNT(e.id) AS "jumlah_mata_kuliah!",
               COALESCE(SUM(mk.sks), 0)::BIGINT AS "total_sks!"
        FROM mahasiswa_pa mp
        LEFT JOIN enrollments e
            ON e.registrasi_id = mp.registrasi_id
           AND e.tahun_akademik_id = $2
        LEFT JOIN jadwal_kuliah jk ON jk.id = e.jadwal_kuliah_id
        LEFT JOIN mata_kuliah mk ON mk.id = jk.matakuliah_id
        GROUP BY mp.id, mp.nim, mp.nama_mahasiswa, mp.angkatan, mp.email,
                 mp.nama_prodi
        ORDER BY mp.nim
        "#,
        dosen_pa_id,
        tahun_akademik_id
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
