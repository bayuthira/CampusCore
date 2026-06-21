// src/modules/matakuliah/repo.rs
use super::model::{
    CreateMataKuliahPayload, MataKuliahDetail, UpdateMataKuliahPayload, VerifikasiRpsPayload,
};
use crate::{db::DbPool, errors::AppError};
use uuid::Uuid;

pub async fn create_matakuliah_repo(
    pool: &DbPool,
    payload: CreateMataKuliahPayload,
) -> Result<MataKuliahDetail, AppError> {
    // Backend menghitung Total SKS otomatis
    let total_sks = payload.sks_tatap_muka
        + payload.sks_praktek
        + payload.sks_praktek_lapangan
        + payload.sks_simulasi;

    let jenis = payload.jenis_mk.unwrap_or_else(|| "Wajib".to_string());

    let mk_id = sqlx::query_scalar!(
        r#"
        INSERT INTO mata_kuliah (
            kode_mk, nama_mk, sks, semester_target, prodi_id,
            id_matkul_feeder, sks_tatap_muka, sks_praktek, sks_praktek_lapangan, sks_simulasi, jenis_mk,
            status_verifikasi_rps
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'Belum Upload') RETURNING id
        "#,
        payload.kode_mk, payload.nama_mk, total_sks, payload.semester_target, payload.prodi_id,
        payload.id_matkul_feeder, payload.sks_tatap_muka, payload.sks_praktek,
        payload.sks_praktek_lapangan, payload.sks_simulasi, jenis
    )
    .fetch_one(pool)
    .await?;

    let new_mk = get_matakuliah_by_id_repo(pool, mk_id).await?;
    Ok(new_mk)
}

pub async fn get_all_matakuliah_repo(pool: &DbPool) -> Result<Vec<MataKuliahDetail>, AppError> {
    let mk_list = sqlx::query_as!(
        MataKuliahDetail,
        r#"
        SELECT mk.id, mk.kode_mk, mk.nama_mk, mk.sks, mk.semester_target, mk.prodi_id, p.nama_prodi,
               mk.id_matkul_feeder, mk.sks_tatap_muka, mk.sks_praktek, mk.sks_praktek_lapangan, mk.sks_simulasi, mk.jenis_mk,
               mk.file_rps_path, mk.status_verifikasi_rps, mk.catatan_verifikasi_rps
        FROM mata_kuliah mk
        LEFT JOIN prodi p ON mk.prodi_id = p.id
        ORDER BY mk.kode_mk ASC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(mk_list)
}

pub async fn get_matakuliah_by_id_repo(
    pool: &DbPool,
    id: Uuid,
) -> Result<MataKuliahDetail, AppError> {
    let mk = sqlx::query_as!(
        MataKuliahDetail,
        r#"
        SELECT mk.id, mk.kode_mk, mk.nama_mk, mk.sks, mk.semester_target, mk.prodi_id, p.nama_prodi,
               mk.id_matkul_feeder, mk.sks_tatap_muka, mk.sks_praktek, mk.sks_praktek_lapangan, mk.sks_simulasi, mk.jenis_mk,
               mk.file_rps_path, mk.status_verifikasi_rps, mk.catatan_verifikasi_rps
        FROM mata_kuliah mk
        LEFT JOIN prodi p ON mk.prodi_id = p.id
        WHERE mk.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(mk)
}

pub async fn update_matakuliah_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateMataKuliahPayload,
) -> Result<MataKuliahDetail, AppError> {
    // Mulai transaksi
    let mut tx = pool.begin().await?;

    // 1. Ambil data lama
    let old_mk = get_matakuliah_by_id_repo(pool, id).await?;

    // 2. Tentukan pecahan SKS baru (jika tidak dikirim FE, pakai data lama)
    let upd_tatap_muka = payload.sks_tatap_muka.unwrap_or(old_mk.sks_tatap_muka);
    let upd_praktek = payload.sks_praktek.unwrap_or(old_mk.sks_praktek);
    let upd_lapangan = payload
        .sks_praktek_lapangan
        .unwrap_or(old_mk.sks_praktek_lapangan);
    let upd_simulasi = payload.sks_simulasi.unwrap_or(old_mk.sks_simulasi);

    // 3. Hitung ulang total SKS
    let upd_total_sks = upd_tatap_muka + upd_praktek + upd_lapangan + upd_simulasi;

    let upd_kode = payload.kode_mk.unwrap_or(old_mk.kode_mk);
    let upd_nama = payload.nama_mk.unwrap_or(old_mk.nama_mk);
    let upd_semester = payload.semester_target.unwrap_or(old_mk.semester_target);
    let upd_prodi = payload.prodi_id.unwrap_or(old_mk.prodi_id);
    let upd_feeder = payload.id_matkul_feeder.or(old_mk.id_matkul_feeder);
    let upd_jenis = payload.jenis_mk.unwrap_or(old_mk.jenis_mk);

    // 4. Lakukan Update
    sqlx::query!(
        r#"
        UPDATE mata_kuliah SET 
            kode_mk = $1, nama_mk = $2, sks = $3, semester_target = $4, prodi_id = $5,
            id_matkul_feeder = $6, sks_tatap_muka = $7, sks_praktek = $8, 
            sks_praktek_lapangan = $9, sks_simulasi = $10, jenis_mk = $11, updated_at = now()
        WHERE id = $12
        "#,
        upd_kode,
        upd_nama,
        upd_total_sks,
        upd_semester,
        upd_prodi,
        upd_feeder,
        upd_tatap_muka,
        upd_praktek,
        upd_lapangan,
        upd_simulasi,
        upd_jenis,
        id
    )
    .execute(&mut *tx)
    .await?;

    // 5. Commit semua perubahan
    tx.commit().await?;

    // Ambil dan kembalikan data terbaru
    let updated_mk = get_matakuliah_by_id_repo(pool, id).await?;
    Ok(updated_mk)
}

pub async fn delete_matakuliah_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM mata_kuliah WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}

// --- FUNGSI VERIFIKASI RPS ---
pub async fn verifikasi_rps_repo(
    pool: &DbPool,
    id: Uuid,
    payload: VerifikasiRpsPayload,
) -> Result<MataKuliahDetail, AppError> {
    if payload.status_verifikasi != "Disetujui" && payload.status_verifikasi != "Ditolak" {
        return Err(AppError::BadRequest(
            "Status verifikasi harus 'Disetujui' atau 'Ditolak'.".to_string(),
        ));
    }

    let (file_rps_path, current_status) = sqlx::query_as::<_, (Option<String>, Option<String>)>(
        "SELECT file_rps_path, status_verifikasi_rps FROM mata_kuliah WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;

    if file_rps_path.is_none() {
        return Err(AppError::BadRequest(
            "Dokumen RPS harus diunggah sebelum diverifikasi.".to_string(),
        ));
    }

    if current_status.as_deref() != Some("Menunggu Verifikasi") {
        return Err(AppError::BadRequest(
            "Hanya RPS berstatus 'Menunggu Verifikasi' yang dapat diverifikasi.".to_string(),
        ));
    }

    let rows_affected = sqlx::query!(
        r#"
        UPDATE mata_kuliah 
        SET status_verifikasi_rps = $1, catatan_verifikasi_rps = $2, updated_at = now()
        WHERE id = $3
        "#,
        payload.status_verifikasi,
        payload.catatan,
        id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    let updated_mk = get_matakuliah_by_id_repo(pool, id).await?;
    Ok(updated_mk)
}

pub async fn update_file_rps_repo(
    pool: &DbPool,
    id: Uuid,
    file_path: String,
) -> Result<Option<String>, AppError> {
    let old_path = sqlx::query_scalar::<_, Option<String>>(
        "SELECT file_rps_path FROM mata_kuliah WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(sqlx::Error::RowNotFound)?;

    let rows_affected = sqlx::query!(
        r#"
        UPDATE mata_kuliah 
        SET file_rps_path = $1, status_verifikasi_rps = 'Menunggu Verifikasi',
            catatan_verifikasi_rps = NULL, updated_at = now()
        WHERE id = $2
        "#,
        file_path,
        id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }

    Ok(old_path)
}
