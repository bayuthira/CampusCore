// src/modules/kurikulum/repo.rs
use crate::{db::DbPool, errors::AppError, modules::matakuliah::model::MataKuliahDetail};
use uuid::Uuid;

use super::model::{
    AddMataKuliahToKurikulumPayload, CreateKurikulumPayload, KurikulumDetail, MappingCsvRow,
    UpdateKurikulumPayload,
};

pub async fn get_kurikulum_by_id_repo_inner(
    pool: &DbPool,
    id: Uuid,
) -> Result<KurikulumDetail, AppError> {
    let kurikulum = sqlx::query_as!(
        KurikulumDetail,
        r#"
        SELECT k.*, p.nama_prodi FROM kurikulum k
        LEFT JOIN prodi p ON k.prodi_id = p.id
        WHERE k.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(kurikulum)
}

pub async fn create_kurikulum_repo(
    pool: &DbPool,
    payload: CreateKurikulumPayload,
) -> Result<KurikulumDetail, AppError> {
    let sks_lulus = payload.sks_lulus.unwrap_or(144); // Standar S1 = 144
    let sks_wajib = payload.sks_wajib.unwrap_or(0);
    let sks_pilihan = payload.sks_pilihan.unwrap_or(0);

    let id = sqlx::query_scalar!(
        r#"
        INSERT INTO kurikulum (
            nama, tahun_mulai, is_active, prodi_id, 
            id_kurikulum_feeder, sks_lulus, sks_wajib, sks_pilihan, id_semester_mulai
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id
        "#,
        payload.nama,
        payload.tahun_mulai,
        payload.is_active,
        payload.prodi_id,
        payload.id_kurikulum_feeder,
        sks_lulus,
        sks_wajib,
        sks_pilihan,
        payload.id_semester_mulai
    )
    .fetch_one(pool)
    .await?;

    let new_kurikulum = get_kurikulum_by_id_repo_inner(pool, id).await?;
    Ok(new_kurikulum)
}

pub async fn get_all_kurikulum_repo(pool: &DbPool) -> Result<Vec<KurikulumDetail>, AppError> {
    let kurikulum_list = sqlx::query_as!(
        KurikulumDetail,
        r#"
        SELECT k.*, COALESCE(p.nama_prodi, 'Prodi Tidak Ditemukan') as "nama_prodi!"
        FROM kurikulum k
        LEFT JOIN prodi p ON k.prodi_id = p.id
        ORDER BY k.tahun_mulai DESC, k.nama ASC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(kurikulum_list)
}

pub async fn update_kurikulum_repo(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateKurikulumPayload,
) -> Result<KurikulumDetail, AppError> {
    let mut tx = pool.begin().await?;

    // Ambil data lama
    let old = get_kurikulum_by_id_repo_inner(pool, id).await?;

    let upd_nama = payload.nama.unwrap_or(old.nama);
    let upd_tahun = payload.tahun_mulai.unwrap_or(old.tahun_mulai);
    let upd_active = payload.is_active.unwrap_or(old.is_active);
    let upd_prodi = payload.prodi_id.unwrap_or(old.prodi_id);
    let upd_feeder = payload.id_kurikulum_feeder.or(old.id_kurikulum_feeder);
    let upd_lulus = payload.sks_lulus.unwrap_or(old.sks_lulus);
    let upd_wajib = payload.sks_wajib.unwrap_or(old.sks_wajib);
    let upd_pilihan = payload.sks_pilihan.unwrap_or(old.sks_pilihan);
    let upd_smt_mulai = payload.id_semester_mulai.or(old.id_semester_mulai);

    sqlx::query!(
        r#"
        UPDATE kurikulum SET 
            nama = $1, tahun_mulai = $2, is_active = $3, prodi_id = $4, 
            id_kurikulum_feeder = $5, sks_lulus = $6, sks_wajib = $7, sks_pilihan = $8,
            id_semester_mulai = $9, updated_at = now() 
        WHERE id = $10
        "#,
        upd_nama,
        upd_tahun,
        upd_active,
        upd_prodi,
        upd_feeder,
        upd_lulus,
        upd_wajib,
        upd_pilihan,
        upd_smt_mulai,
        id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let updated_kurikulum = get_kurikulum_by_id_repo_inner(pool, id).await?;
    Ok(updated_kurikulum)
}

pub async fn delete_kurikulum_repo(pool: &DbPool, id: Uuid) -> Result<(), AppError> {
    let rows_affected = sqlx::query!("DELETE FROM kurikulum WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected();
    if rows_affected == 0 {
        return Err(sqlx::Error::RowNotFound.into());
    }
    Ok(())
}

pub async fn add_matakuliah_to_kurikulum_repo(
    pool: &DbPool,
    kurikulum_id: Uuid,
    payload: AddMataKuliahToKurikulumPayload,
) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO kurikulum_matakuliah (kurikulum_id, matakuliah_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        kurikulum_id, payload.matakuliah_id
    ).execute(pool).await?;
    Ok(())
}

pub async fn get_matakuliah_in_kurikulum_repo(
    pool: &DbPool,
    kurikulum_id: Uuid,
) -> Result<Vec<MataKuliahDetail>, AppError> {
    // Memperbaiki error E0063 dengan menambahkan kolom Feeder Mata Kuliah yang kurang
    let mk_list = sqlx::query_as!(
        MataKuliahDetail,
        r#"
        SELECT 
            mk.id, mk.kode_mk, mk.nama_mk, mk.sks, mk.semester_target, mk.prodi_id,
            mk.id_matkul_feeder, mk.sks_tatap_muka, mk.sks_praktek, mk.sks_praktek_lapangan, mk.sks_simulasi, mk.jenis_mk,
            COALESCE(p.nama_prodi, 'Prodi Tidak Ditemukan') as "nama_prodi!"
        FROM mata_kuliah mk
        INNER JOIN kurikulum_matakuliah km ON mk.id = km.matakuliah_id
        LEFT JOIN prodi p ON mk.prodi_id = p.id
        WHERE km.kurikulum_id = $1
        ORDER BY mk.kode_mk
        "#,
        kurikulum_id
    )
    .fetch_all(pool)
    .await?;
    Ok(mk_list)
}

pub async fn remove_matakuliah_from_kurikulum_repo(
    pool: &DbPool,
    kurikulum_id: Uuid,
    matakuliah_id: Uuid,
) -> Result<(), AppError> {
    sqlx::query!(
        "DELETE FROM kurikulum_matakuliah WHERE kurikulum_id = $1 AND matakuliah_id = $2",
        kurikulum_id,
        matakuliah_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Fungsi untuk mengeksekusi import CSV pemetaan mata kuliah ke kurikulum
pub async fn import_mapping_csv_repo(
    pool: &DbPool,
    records: Vec<MappingCsvRow>,
) -> Result<(usize, usize, Vec<String>), AppError> {
    let mut tx = pool.begin().await?;

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut error_messages = Vec::new();

    for (index, row) in records.into_iter().enumerate() {
        let row_number = index + 2; // +2 karena baris 1 adalah header

        // 1. Cari ID Kurikulum berdasarkan Namanya
        let kurikulum = sqlx::query!(
            "SELECT id FROM kurikulum WHERE nama = $1",
            row.nama_kurikulum
        )
        .fetch_optional(&mut *tx)
        .await?;

        // 2. Cari ID Mata Kuliah berdasarkan Kode MK-nya
        let matakuliah = sqlx::query!("SELECT id FROM mata_kuliah WHERE kode_mk = $1", row.kode_mk)
            .fetch_optional(&mut *tx)
            .await?;

        // 3. Validasi & Insert
        match (kurikulum, matakuliah) {
            (Some(k), Some(mk)) => {
                // INSERT dengan ON CONFLICT DO NOTHING agar jika data sudah ada, tidak terjadi error DB
                // Asumsi nama tabel relasi Anda adalah `kurikulum_matakuliah`
                let _result_ = sqlx::query!(
                    r#"
                    INSERT INTO kurikulum_matakuliah (kurikulum_id, matakuliah_id) 
                    VALUES ($1, $2) 
                    ON CONFLICT DO NOTHING
                    "#,
                    k.id,
                    mk.id
                )
                .execute(&mut *tx)
                .await?;

                // Jika rows_affected 0, artinya sudah pernah ditambahkan sebelumnya, tetap kita anggap sukses
                success_count += 1;
            }
            (None, _) => {
                failed_count += 1;
                error_messages.push(format!(
                    "Baris {}: Kurikulum '{}' tidak ditemukan.",
                    row_number, row.nama_kurikulum
                ));
            }
            (_, None) => {
                failed_count += 1;
                error_messages.push(format!(
                    "Baris {}: Mata Kuliah dengan kode '{}' tidak ditemukan.",
                    row_number, row.kode_mk
                ));
            }
        }
    }

    tx.commit().await?;

    Ok((success_count, failed_count, error_messages))
}
